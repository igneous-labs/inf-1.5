use inf1_core::{instructions::swap::IxAccs, quote::Quote};
use inf1_ctl_jiminy::{
    account_utils::{
        lst_state_list_checked, lst_state_list_get, pool_state_v2_checked,
        pool_state_v2_checked_mut,
    },
    cpi::{PricingRetVal, SolValCalcRetVal},
    err::Inf1CtlErr,
    instructions::{
        swap::{
            v2::{IxPreAccs, NewIxPreAccsBuilder},
            IxArgs,
        },
        sync_sol_value::NewSyncSolValueIxPreAccsBuilder,
    },
    keys::{LST_STATE_LIST_ID, POOL_STATE_ID},
    pda_onchain::{create_raw_pool_reserves_addr, POOL_STATE_SIGNER},
    program_err::Inf1CtlCustomProgErr,
    sync_sol_val::SyncSolVal,
    typedefs::{
        lst_state::LstState,
        pool_sv::{PoolSvLamports, PoolSvMutRefs},
        snap::{NewSnapBuilder, SnapU64},
        u8bool::U8Bool,
    },
    yields::update::UpdateYield,
};
use jiminy_cpi::{
    account::{Abr, AccountHandle},
    program_error::{ProgramError, NOT_ENOUGH_ACCOUNT_KEYS},
};
use jiminy_sysvar_clock::Clock;
use sanctum_spl_token_jiminy::{
    instructions::{
        mint_to::mint_to_ix_account_handle_perms,
        transfer::transfer_checked_ix_account_handle_perms,
    },
    sanctum_spl_token_core::instructions::{
        burn::{BurnIxData, NewBurnIxAccsBuilder},
        mint_to::{MintToIxData, NewMintToIxAccsBuilder},
        transfer::{NewTransferCheckedIxAccsBuilder, TransferCheckedIxData},
    },
};
use sanctum_u64_ratio::Ratio;

use crate::{
    acc_migrations::pool_state,
    svc::{
        cpi_lst_reserves_sol_val, lst_sync_sol_val, update_lst_state_sol_val, SyncSolValIxAccounts,
    },
    token::checked_mint_of,
    verify::{verify_not_rebalancing_and_not_disabled_v2, verify_pks},
    Cpi,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SwapCpiRetVals {
    pub inp_calc: SolValCalcRetVal,
    pub out_calc: SolValCalcRetVal,
    pub pricing: PricingRetVal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SwapV2Ctl<Swap, AddLiq, RemLiq> {
    Swap(Swap),
    AddLiq(AddLiq),
    RemLiq(RemLiq),
}

pub type SwapV2IxAccounts<'a, 'acc> = IxAccs<
    [u8; 32], // program accs are pubkeys instead of AccountHandles to be compatible with v1 liquidity instructions
    &'a IxPreAccs<AccountHandle<'acc>>,
    &'a [AccountHandle<'acc>],
    &'a [AccountHandle<'acc>],
    &'a [AccountHandle<'acc>],
>;

type SwapV2CtlUni<T> = SwapV2Ctl<T, T, T>;

impl<T> AsRef<T> for SwapV2CtlUni<T> {
    #[inline]
    fn as_ref(&self) -> &T {
        match self {
            Self::Swap(t) => t,
            Self::AddLiq(t) => t,
            Self::RemLiq(t) => t,
        }
    }
}

pub type SwapV2CtlIxAccounts<'a, 'acc> = SwapV2CtlUni<SwapV2IxAccounts<'a, 'acc>>;

#[inline]
pub fn swap_v2_checked<'a, 'acc>(
    abr: &mut Abr,
    ix_prefix: &'a IxPreAccs<AccountHandle<'acc>>,
    suf: &'a [AccountHandle<'acc>],
    args: &IxArgs,
    clock: &Clock,
) -> Result<SwapV2CtlIxAccounts<'a, 'acc>, ProgramError> {
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

    let [Some((inp_calc_prog, inp_calc)), Some((out_calc_prog, out_calc)), Some((pricing_prog, pricing))] =
        [inp_calc_all, out_calc_all, pricing_all].map(|a| a.split_first())
    else {
        return Err(NOT_ENOUGH_ACCOUNT_KEYS.into());
    };

    verify_pks(abr, &[*pricing_prog], &[&pool.pricing_program])?;

    let [i, o]: [Result<_, ProgramError>; 2] = [
        (args.inp_lst_index, ix_prefix.inp_mint(), inp_calc_prog),
        (args.out_lst_index, ix_prefix.out_mint(), out_calc_prog),
    ]
    .map(|(idx, mint_handle, calc_prog)| {
        Ok(match idx {
            u32::MAX => (
                pool.lp_token_mint,
                abr.get(*mint_handle).owner(),
                &pool.lp_token_mint,
                false,
                // no verification of calc_prog for INF token;
                // can be any filler account since its not used
            ),
            idx => {
                let LstState {
                    pool_reserves_bump,
                    mint,
                    sol_value_calculator,
                    is_input_disabled,
                    ..
                } = lst_state_list_get(list, idx as usize)?;
                let token_prog = abr.get(*mint_handle).owner();
                let reserves = create_raw_pool_reserves_addr(token_prog, mint, pool_reserves_bump)
                    .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidReserves))?;

                verify_pks(abr, &[*calc_prog], &[sol_value_calculator])?;

                (
                    reserves,
                    token_prog,
                    mint,
                    U8Bool(is_input_disabled).to_bool(),
                )
            }
        })
    });
    let (expected_inp_reserves, expected_inp_token_prog, expected_inp_mint, is_inp_disabled) = i?;
    let (expected_out_reserves, expected_out_token_prog, expected_out_mint, _) = o?;

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

    let [inp_calc_prog, out_calc_prog] = [inp_calc_prog, out_calc_prog].map(|h| *abr.get(*h).key());

    let accs = SwapV2IxAccounts {
        ix_prefix,
        inp_calc_prog,
        inp_calc,
        out_calc_prog,
        out_calc,
        pricing_prog: pool.pricing_program,
        pricing,
    };

    let accs = if args.inp_lst_index == u32::MAX {
        SwapV2CtlIxAccounts::RemLiq(accs)
    } else if args.out_lst_index == u32::MAX {
        SwapV2CtlIxAccounts::AddLiq(accs)
    } else {
        SwapV2CtlIxAccounts::Swap(accs)
    };

    Ok(accs)
}

/// Returns [inp, out].
///
/// Returned val is not usable if its of the INF mint, ie
/// inp for RemLiq and out for AddLiq
#[inline]
fn sync_pair_accs<'a, 'acc>(
    SwapV2IxAccounts {
        ix_prefix,
        inp_calc,
        inp_calc_prog,
        out_calc,
        out_calc_prog,
        ..
    }: &SwapV2IxAccounts<'a, 'acc>,
    IxArgs {
        inp_lst_index,
        out_lst_index,
        ..
    }: &IxArgs,
) -> [(SyncSolValIxAccounts<'a, 'acc>, usize); 2] {
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
        calc_prog: *calc_prog,
        calc,
    });
    let [inp_lst_index, out_lst_index] = [inp_lst_index, out_lst_index].map(|x| *x as usize);
    [(inp_accs, inp_lst_index), (out_accs, out_lst_index)]
}

/// TODO: use return value to create yield update event for self-cpi logging
#[inline]
pub fn initial_sync(
    abr: &mut Abr,
    cpi: &mut Cpi,
    accs: &SwapV2CtlIxAccounts,
    args: &IxArgs,
) -> Result<(), ProgramError> {
    let [(inp_accs, inp_lst_index), (out_accs, out_lst_index)] =
        sync_pair_accs(accs.as_ref(), args);
    match accs {
        SwapV2Ctl::Swap(_) => [(inp_accs, inp_lst_index), (out_accs, out_lst_index)]
            .into_iter()
            .try_for_each(|(a, i)| lst_sync_sol_val(abr, cpi, &a, i)),
        SwapV2Ctl::AddLiq(_) => lst_sync_sol_val(abr, cpi, &inp_accs, inp_lst_index),
        SwapV2Ctl::RemLiq(_) => lst_sync_sol_val(abr, cpi, &out_accs, out_lst_index),
    }
}

#[inline]
pub fn move_tokens(
    abr: &mut Abr,
    cpi: &mut Cpi,
    accs: &SwapV2CtlIxAccounts,
    Quote { inp, out, .. }: &Quote,
) -> Result<(), ProgramError> {
    match accs {
        SwapV2Ctl::RemLiq(accs) => cpi.invoke_fwd_handle(
            abr,
            *accs.ix_prefix.inp_token_program(),
            BurnIxData::new(*inp).as_buf(),
            NewBurnIxAccsBuilder::start()
                .with_auth(*accs.ix_prefix.signer())
                .with_from(*accs.ix_prefix.inp_acc())
                // use inp_pool_reserves instead of inp_mint
                // to get write permission
                .with_mint(*accs.ix_prefix.inp_pool_reserves())
                .build()
                .0,
        ),
        SwapV2Ctl::AddLiq(accs) | SwapV2Ctl::Swap(accs) => cpi.invoke_fwd_handle(
            abr,
            *accs.ix_prefix.inp_token_program(),
            TransferCheckedIxData::new(
                *inp,
                checked_mint_of(abr.get(*accs.ix_prefix.inp_mint()))?.decimals(),
            )
            .as_buf(),
            NewTransferCheckedIxAccsBuilder::start()
                .with_auth(*accs.ix_prefix.signer())
                .with_src(*accs.ix_prefix.inp_acc())
                .with_dst(*accs.ix_prefix.inp_pool_reserves())
                .with_mint(*accs.ix_prefix.inp_mint())
                .build()
                .0,
        ),
    }?;
    match accs {
        SwapV2Ctl::AddLiq(accs) => cpi.invoke_signed_handle(
            abr,
            *accs.ix_prefix.out_token_program(),
            MintToIxData::new(*out).as_buf(),
            mint_to_ix_account_handle_perms(
                NewMintToIxAccsBuilder::start()
                    .with_auth(*accs.ix_prefix.pool_state())
                    // use out_pool_reserves instead of inp_mint
                    // to get write permission
                    .with_mint(*accs.ix_prefix.out_pool_reserves())
                    .with_to(*accs.ix_prefix.out_acc())
                    .build(),
            ),
            &[POOL_STATE_SIGNER],
        ),
        SwapV2Ctl::RemLiq(accs) | SwapV2Ctl::Swap(accs) => cpi.invoke_signed_handle(
            abr,
            *accs.ix_prefix.out_token_program(),
            TransferCheckedIxData::new(
                *out,
                checked_mint_of(abr.get(*accs.ix_prefix.out_mint()))?.decimals(),
            )
            .as_buf(),
            transfer_checked_ix_account_handle_perms(
                NewTransferCheckedIxAccsBuilder::start()
                    .with_auth(*accs.ix_prefix.pool_state())
                    .with_src(*accs.ix_prefix.out_pool_reserves())
                    .with_dst(*accs.ix_prefix.out_acc())
                    .with_mint(*accs.ix_prefix.out_mint())
                    .build(),
            ),
            &[POOL_STATE_SIGNER],
        ),
    }?;
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LiqFinalSync {
    pub inf_supply: SnapU64,
}

pub type SwapV2FinalSyncAux = SwapV2Ctl<(), LiqFinalSync, LiqFinalSync>;

/// Contained u64 is INF mint supply
pub type SwapV2FinalSyncAuxPre = SwapV2Ctl<(), u64, u64>;

#[inline]
pub fn final_sync_aux_pre_movement(
    abr: &Abr,
    accs: &SwapV2CtlIxAccounts,
) -> Result<SwapV2FinalSyncAuxPre, ProgramError> {
    let (inf_mint_handle, ctor) = match accs {
        SwapV2Ctl::Swap(_) => return Ok(SwapV2Ctl::Swap(())),
        SwapV2Ctl::AddLiq(accs) => (
            accs.ix_prefix.out_mint(),
            SwapV2Ctl::AddLiq as fn(u64) -> SwapV2FinalSyncAuxPre,
        ),
        SwapV2Ctl::RemLiq(accs) => (accs.ix_prefix.inp_mint(), SwapV2Ctl::RemLiq as _),
    };
    Ok(ctor(checked_mint_of(abr.get(*inf_mint_handle))?.supply()))
}

#[inline]
pub fn final_sync_aux_post_movement(
    abr: &Abr,
    ix_prefix: &IxPreAccs<AccountHandle<'_>>,
    pre: SwapV2FinalSyncAuxPre,
) -> Result<SwapV2FinalSyncAux, ProgramError> {
    let (old_inf_supply, inf_mint_handle, ctor) = match pre {
        SwapV2Ctl::Swap(_) => return Ok(SwapV2Ctl::Swap(())),
        SwapV2Ctl::AddLiq(old_inf_supply) => (
            old_inf_supply,
            ix_prefix.out_mint(),
            SwapV2Ctl::AddLiq as fn(LiqFinalSync) -> SwapV2FinalSyncAux,
        ),
        SwapV2Ctl::RemLiq(old_inf_supply) => {
            (old_inf_supply, ix_prefix.inp_mint(), SwapV2Ctl::RemLiq as _)
        }
    };
    Ok(ctor(LiqFinalSync {
        inf_supply: NewSnapBuilder::start()
            .with_old(old_inf_supply)
            .with_new(checked_mint_of(abr.get(*inf_mint_handle))?.supply())
            .build(),
    }))
}

/// TODO: use return value to create yield update event for self-cpi logging
#[inline]
pub fn final_sync(
    abr: &mut Abr,
    cpi: &mut Cpi,
    accs: &SwapV2IxAccounts,
    args: &IxArgs,
    aux: &SwapV2FinalSyncAux,
) -> Result<(), ProgramError> {
    let [inp, out] = sync_pair_accs(accs, args);
    let ((lst_accs, lst_idx), inf_supply) = match aux {
        SwapV2Ctl::Swap(_) => {
            let [inp, out] = [inp, out].map(|(accs, lst_idx)| {
                let lst_new = cpi_lst_reserves_sol_val(abr, cpi, &accs)?;
                update_lst_state_sol_val(abr, *accs.ix_prefix.lst_state_list(), lst_idx, lst_new)
            });
            let inp_snap = inp?;
            let out_snap = out?;

            let pool = pool_state_v2_checked_mut(abr.get_mut(*accs.ix_prefix.pool_state()))?;

            let [out_sync, inp_sync] =
                [out_snap, inp_snap].map(|lst_sol_val| SyncSolVal { lst_sol_val });
            // exec on out first to reduce odds of overflow
            // since out will be a decrease
            let new_total_sol_value = out_sync
                .exec(pool.total_sol_value)
                .and_then(|pool_total_sol_value| inp_sync.exec(pool_total_sol_value))
                .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::MathError))?;

            if new_total_sol_value < pool.total_sol_value {
                return Err(Inf1CtlCustomProgErr(Inf1CtlErr::PoolWouldLoseSolValue).into());
            }

            let new = UpdateYield {
                new_total_sol_value,
                old: PoolSvLamports::from_pool_state_v2(pool),
            }
            .exec()
            .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::MathError))?;
            PoolSvMutRefs::from_pool_state_v2(pool).update(new);
            return Ok(());
        }
        SwapV2Ctl::AddLiq(LiqFinalSync { inf_supply }) => (inp, inf_supply),
        SwapV2Ctl::RemLiq(LiqFinalSync { inf_supply }) => (out, inf_supply),
    };

    let lst_new = cpi_lst_reserves_sol_val(abr, cpi, &lst_accs)?;
    let lst_sol_val =
        update_lst_state_sol_val(abr, *accs.ix_prefix.lst_state_list(), lst_idx, lst_new)?;

    let pool = pool_state_v2_checked_mut(abr.get_mut(*accs.ix_prefix.pool_state()))?;

    let new_total_sol_value = SyncSolVal { lst_sol_val }
        .exec(pool.total_sol_value)
        .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::MathError))?;

    verify_liq_no_loss(
        &NewSnapBuilder::start()
            .with_old(pool.total_sol_value)
            .with_new(new_total_sol_value)
            .build(),
        inf_supply,
    )?;

    let new = UpdateYield {
        new_total_sol_value,
        old: PoolSvLamports::from_pool_state_v2(pool),
    }
    .normalized(inf_supply)
    .and_then(|uy| uy.exec())
    .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::MathError))?;
    PoolSvMutRefs::from_pool_state_v2(pool).update(new);
    Ok(())
}

/// Used by add/remove liquidity to ensure that redemption rate
/// does not go down after the instruction
#[inline]
fn verify_liq_no_loss(
    total_sol_value: &SnapU64,
    inf_supply: &SnapU64,
) -> Result<(), Inf1CtlCustomProgErr> {
    let [old_r, new_r] = [
        (*total_sol_value.old(), *inf_supply.old()),
        (*total_sol_value.new(), *inf_supply.new()),
    ]
    .map(|(n, d)| Ratio { n, d });
    if new_r < old_r {
        Err(Inf1CtlCustomProgErr(Inf1CtlErr::PoolWouldLoseSolValue))
    } else {
        Ok(())
    }
}
