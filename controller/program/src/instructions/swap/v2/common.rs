use inf1_core::instructions::swap::IxAccs;
use inf1_ctl_jiminy::{
    account_utils::{lst_state_list_checked, lst_state_list_get, pool_state_v2_checked},
    err::Inf1CtlErr,
    instructions::{
        swap::{
            v2::{IxPreAccs, NewIxPreAccsBuilder},
            IxArgs,
        },
        sync_sol_value::NewSyncSolValueIxPreAccsBuilder,
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
    svc::{lst_sync_sol_val, SyncSolValIxAccounts},
    verify::{verify_not_rebalancing_and_not_disabled_v2, verify_pks},
    Cpi,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SwapV2Ctl<Swap, AddLiq, RemLiq> {
    Swap(Swap),
    AddLiq(AddLiq),
    RemLiq(RemLiq),
}

pub type SwapV2Ty = SwapV2Ctl<(), (), ()>;

pub type SwapV2IxAccounts<'a, 'acc> = IxAccs<
    Option<[u8; 32]>, // program accs made optional to be compatible with v1 liquidity instructions
    &'a IxPreAccs<AccountHandle<'acc>>,
    &'a [AccountHandle<'acc>],
    &'a [AccountHandle<'acc>],
    &'a [AccountHandle<'acc>],
>;

#[inline]
pub fn swap_v2_checked<'a, 'acc>(
    abr: &mut Abr,
    ix_prefix: &'a IxPreAccs<AccountHandle<'acc>>,
    suf: &'a [AccountHandle<'acc>],
    args: &IxArgs,
    clock: &Clock,
) -> Result<(SwapV2IxAccounts<'a, 'acc>, SwapV2Ty), ProgramError> {
    if args.amount == 0 {
        return Err(Inf1CtlCustomProgErr(Inf1CtlErr::ZeroValue).into());
    }

    pool_state::v2::migrate_idmpt(abr.get_mut(*ix_prefix.pool_state()), clock)?;

    let pool = pool_state_v2_checked(abr.get(*ix_prefix.pool_state()))?;

    verify_not_rebalancing_and_not_disabled_v2(pool)?;

    let list = lst_state_list_checked(abr.get(*ix_prefix.lst_state_list()))?;

    let (inp_calc_all, suf) = suf
        .split_at_checked(args.inp_lst_value_calc_accs.into())
        .ok_or(NOT_ENOUGH_ACCOUNT_KEYS)?;
    let (out_calc_all, pricing_all) = suf
        .split_at_checked(args.out_lst_value_calc_accs.into())
        .ok_or(NOT_ENOUGH_ACCOUNT_KEYS)?;

    let (pricing_prog, pricing) = pricing_all.split_first().ok_or(NOT_ENOUGH_ACCOUNT_KEYS)?;
    verify_pks(abr, &[*pricing_prog], &[&pool.pricing_program])?;

    let [i, o]: [Result<_, ProgramError>; 2] = [
        (args.inp_lst_index, ix_prefix.inp_mint(), inp_calc_all),
        (args.out_lst_index, ix_prefix.out_mint(), out_calc_all),
    ]
    .map(|(i, mint_handle, calc_all)| {
        Ok(match i {
            u32::MAX => (
                pool.lp_token_mint,
                abr.get(*mint_handle).owner(),
                &pool.lp_token_mint,
                None,
                calc_all,
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
                let (calc_prog, calc) = calc_all.split_first().ok_or(NOT_ENOUGH_ACCOUNT_KEYS)?;

                verify_pks(abr, &[*calc_prog], &[sol_value_calculator])?;

                (
                    reserves,
                    token_prog,
                    mint,
                    Some(calc_prog),
                    calc,
                    U8Bool(is_input_disabled).to_bool(),
                )
            }
        })
    });
    let (
        expected_inp_reserves,
        expected_inp_token_prog,
        expected_inp_mint,
        inp_calc_prog,
        inp_calc,
        is_inp_disabled,
    ) = i?;
    let (
        expected_out_reserves,
        expected_out_token_prog,
        expected_out_mint,
        out_calc_prog,
        out_calc,
        _,
    ) = o?;

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
        // Doesnt matter if its set to pool reserves because we never invoke
        // transfer from inp_acc with signing PDA
        .with_inp_acc(abr.get(*ix_prefix.inp_acc()).key())
        .with_out_acc(abr.get(*ix_prefix.out_acc()).key())
        .with_signer(abr.get(*ix_prefix.signer()).key())
        .build();

    verify_pks(abr, &ix_prefix.0, &expected_pre.0)?;

    let ty = if args.inp_lst_index == u32::MAX {
        SwapV2Ty::RemLiq(())
    } else if args.out_lst_index == u32::MAX {
        SwapV2Ty::AddLiq(())
    } else {
        SwapV2Ty::Swap(())
    };

    let [inp_calc_prog, out_calc_prog] =
        [inp_calc_prog, out_calc_prog].map(|opt| opt.map(|h| *abr.get(*h).key()));

    Ok((
        SwapV2IxAccounts {
            ix_prefix,
            inp_calc_prog,
            inp_calc,
            out_calc_prog,
            out_calc,
            pricing_prog: Some(pool.pricing_program),
            pricing,
        },
        ty,
    ))
}

/// TODO: use return value to create yield update event for self-cpi logging
#[inline]
pub fn initial_pair_sync(
    abr: &mut Abr,
    cpi: &mut Cpi,
    SwapV2IxAccounts {
        ix_prefix,
        inp_calc,
        inp_calc_prog,
        out_calc,
        out_calc_prog,
        ..
    }: &SwapV2IxAccounts,
    IxArgs {
        inp_lst_index,
        out_lst_index,
        ..
    }: &IxArgs,
    ty: SwapV2Ty,
) -> Result<(), ProgramError> {
    let [inp_accs, out_accs] = [
        (
            ix_prefix.inp_mint(),
            ix_prefix.inp_pool_reserves(),
            inp_calc_prog,
            inp_calc,
        ),
        (
            ix_prefix.out_mint(),
            ix_prefix.out_pool_reserves(),
            out_calc_prog,
            out_calc,
        ),
    ]
    .map(|(mint, reserves, calc_prog, calc)| SyncSolValIxAccounts {
        ix_prefix: NewSyncSolValueIxPreAccsBuilder::start()
            .with_pool_state(*ix_prefix.pool_state())
            .with_lst_state_list(*ix_prefix.lst_state_list())
            .with_lst_mint(*mint)
            .with_pool_reserves(*reserves)
            .build(),
        // safety: ty should make it that its unused if None.
        // Even if it does get invoked, its SystemInstruction::CreateAccount
        // with funding = readonly lst mint
        calc_prog: calc_prog.unwrap_or_default(),
        calc,
    });
    let [inp_lst_index, out_lst_index] = [inp_lst_index, out_lst_index].map(|x| *x as usize);
    match ty {
        SwapV2Ty::Swap(_) => [(inp_accs, inp_lst_index), (out_accs, out_lst_index)]
            .into_iter()
            .try_for_each(|(a, i)| lst_sync_sol_val(abr, cpi, &a, i)),
        SwapV2Ty::AddLiq(_) => lst_sync_sol_val(abr, cpi, &inp_accs, inp_lst_index),
        SwapV2Ty::RemLiq(_) => lst_sync_sol_val(abr, cpi, &out_accs, out_lst_index),
    }
}
