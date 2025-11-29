use inf1_core::instructions::swap::IxAccs;
use inf1_ctl_jiminy::{
    account_utils::{lst_state_list_checked, lst_state_list_get, pool_state_v2_checked},
    err::Inf1CtlErr,
    instructions::swap::{
        v2::{IxPreAccs, NewIxPreAccsBuilder},
        IxArgs,
    },
    keys::{LST_STATE_LIST_ID, POOL_STATE_ID},
    pda_onchain::create_raw_pool_reserves_addr,
    program_err::Inf1CtlCustomProgErr,
    typedefs::{lst_state::LstState, u8bool::U8Bool},
};
use jiminy_cpi::{
    account::{Abr, AccountHandle},
    program_error::{ProgramError, NOT_ENOUGH_ACCOUNT_KEYS},
};
use jiminy_sysvar_clock::Clock;

use crate::{
    acc_migrations::pool_state,
    verify::{verify_not_rebalancing_and_not_disabled_v2, verify_pks},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SwapV2Ty {
    Swap,
    AddLiq,
    RemLiq,
}

pub type SwapV2IxAccounts<'a, 'acc> = IxAccs<
    [u8; 32],
    IxPreAccs<AccountHandle<'acc>>,
    &'a [AccountHandle<'acc>],
    &'a [AccountHandle<'acc>],
    &'a [AccountHandle<'acc>],
>;

#[inline]
pub fn swap_v2_checked<'a, 'acc>(
    abr: &mut Abr,
    accounts: &'a [AccountHandle<'acc>],
    args: &IxArgs,
    clock: &Clock,
) -> Result<(SwapV2IxAccounts<'a, 'acc>, SwapV2Ty), ProgramError> {
    if args.amount == 0 {
        return Err(Inf1CtlCustomProgErr(Inf1CtlErr::ZeroValue).into());
    }

    let (ix_prefix, suf) = accounts
        .split_first_chunk()
        .ok_or(NOT_ENOUGH_ACCOUNT_KEYS)?;
    let ix_prefix = IxPreAccs(*ix_prefix);

    pool_state::v2::migrate_idmpt(abr.get_mut(*ix_prefix.pool_state()), clock)?;

    let pool = pool_state_v2_checked(abr.get(*ix_prefix.pool_state()))?;

    verify_not_rebalancing_and_not_disabled_v2(pool)?;

    let list = lst_state_list_checked(abr.get(*ix_prefix.lst_state_list()))?;

    let [i, o]: [Result<_, Inf1CtlCustomProgErr>; 2] = [
        (args.inp_lst_index, ix_prefix.inp_mint()),
        (args.out_lst_index, ix_prefix.out_mint()),
    ]
    .map(|(i, mint_handle)| {
        Ok(match i {
            u32::MAX => (
                pool.lp_token_mint,
                abr.get(*mint_handle).owner(),
                &pool.lp_token_mint,
                &inf1_ctl_jiminy::ID,
                false,
            ),
            i => {
                let LstState {
                    pool_reserves_bump,
                    mint,
                    sol_value_calculator,
                    is_input_disabled,
                    ..
                } = lst_state_list_get(list, i as usize)?;
                let token_prog = abr.get(*mint_handle).owner();
                let reserves = create_raw_pool_reserves_addr(token_prog, mint, pool_reserves_bump)
                    .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidReserves))?;
                (
                    reserves,
                    token_prog,
                    mint,
                    sol_value_calculator,
                    U8Bool(is_input_disabled).to_bool(),
                )
            }
        })
    });
    let (
        expected_inp_reserves,
        expected_inp_token_prog,
        expected_inp_mint,
        expected_inp_svc,
        is_inp_disabled,
    ) = i?;
    let (expected_out_reserves, expected_out_token_prog, expected_out_mint, expected_out_svc, _) =
        o?;

    if is_inp_disabled {
        return Err(Inf1CtlCustomProgErr(Inf1CtlErr::LstInputDisabled).into());
    }

    if expected_inp_mint == expected_out_mint {
        return Err(Inf1CtlCustomProgErr(Inf1CtlErr::SwapSameLst).into());
    }

    let expected_pre = NewIxPreAccsBuilder::start()
        .with_pool_state(&POOL_STATE_ID)
        .with_lst_state_list(&LST_STATE_LIST_ID)
        .with_inp_pool_reserves(&expected_inp_reserves)
        .with_inp_token_program(expected_inp_token_prog)
        .with_inp_mint(expected_inp_mint)
        .with_out_pool_reserves(&expected_out_reserves)
        .with_out_token_program(expected_out_token_prog)
        .with_out_mint(expected_out_mint)
        // Free accounts below: signer is free to specify whatever token
        // accounts to swap from and to.
        // Can't be set to pool reserves because we never invoke
        // transfer from pool reserves with signing PDA
        .with_inp_acc(abr.get(*ix_prefix.inp_acc()).key())
        .with_out_acc(abr.get(*ix_prefix.out_acc()).key())
        .with_signer(abr.get(*ix_prefix.signer()).key())
        .build();

    verify_pks(abr, &ix_prefix.0, &expected_pre.0)?;

    let (inp_calc_all, suf) = suf
        .split_at_checked(args.inp_lst_value_calc_accs.into())
        .ok_or(NOT_ENOUGH_ACCOUNT_KEYS)?;
    let (out_calc_all, pricing_all) = suf
        .split_at_checked(args.out_lst_value_calc_accs.into())
        .ok_or(NOT_ENOUGH_ACCOUNT_KEYS)?;

    let [Some((inp_calc_prog, inp_calc)), Some((out_calc_prog, out_calc)), Some((pricing_prog, pricing))] =
        [inp_calc_all, out_calc_all, pricing_all].map(|arr| arr.split_first())
    else {
        return Err(NOT_ENOUGH_ACCOUNT_KEYS.into());
    };

    verify_pks(
        abr,
        &[*inp_calc_prog, *out_calc_prog, *pricing_prog],
        &[expected_inp_svc, expected_out_svc, &pool.pricing_program],
    )?;

    let ty = if args.inp_lst_index == u32::MAX {
        SwapV2Ty::RemLiq
    } else if args.out_lst_index == u32::MAX {
        SwapV2Ty::AddLiq
    } else {
        SwapV2Ty::Swap
    };

    Ok((
        SwapV2IxAccounts {
            ix_prefix,
            inp_calc_prog: *abr.get(*inp_calc_prog).key(),
            inp_calc,
            out_calc_prog: *abr.get(*out_calc_prog).key(),
            out_calc,
            pricing_prog: *abr.get(*pricing_prog).key(),
            pricing,
        },
        ty,
    ))
}
