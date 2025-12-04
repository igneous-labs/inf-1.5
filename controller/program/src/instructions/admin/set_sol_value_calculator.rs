use inf1_ctl_jiminy::{
    account_utils::{
        lst_state_list_checked, lst_state_list_checked_mut, pool_state_v2_checked,
        pool_state_v2_checked_mut,
    },
    cpi::SetSolValueCalculatorIxPreAccountHandles,
    err::Inf1CtlErr,
    instructions::{
        admin::set_sol_value_calculator::{
            NewSetSolValueCalculatorIxPreAccsBuilder, SetSolValueCalculatorIxPreAccs,
            SET_SOL_VALUE_CALC_IX_PRE_IS_SIGNER,
        },
        sync_sol_value::NewSyncSolValueIxPreAccsBuilder,
    },
    keys::{LST_STATE_LIST_ID, POOL_STATE_ID},
    pda_onchain::create_raw_pool_reserves_addr,
    program_err::Inf1CtlCustomProgErr,
};
use jiminy_cpi::{
    account::{Abr, AccountHandle},
    program_error::{ProgramError, NOT_ENOUGH_ACCOUNT_KEYS},
};

use inf1_core::instructions::admin::set_sol_value_calculator::SetSolValueCalculatorIxAccs;
use inf1_core::instructions::sync_sol_value::SyncSolValueIxAccs;
use jiminy_sysvar_clock::Clock;

use crate::{
    svc::lst_ssv_uy,
    utils::split_suf_accs,
    verify::{
        verify_not_rebalancing_and_not_disabled, verify_pks, verify_signers,
        verify_sol_value_calculator_is_program,
    },
    Cpi,
};

pub type SetSolValueCalculatorIxAccounts<'a, 'acc> = SetSolValueCalculatorIxAccs<
    AccountHandle<'acc>,
    SetSolValueCalculatorIxPreAccountHandles<'acc>,
    &'a [AccountHandle<'acc>],
>;

/// Returns (prefix, sol_val_calc_program, remaining accounts)
#[inline]
pub fn set_sol_value_calculator_accs_checked<'a, 'acc>(
    abr: &Abr,
    accs: &'a [AccountHandle<'acc>],
    lst_idx: usize,
) -> Result<SetSolValueCalculatorIxAccounts<'a, 'acc>, ProgramError> {
    let (ix_prefix, suf) = accs.split_first_chunk().ok_or(NOT_ENOUGH_ACCOUNT_KEYS)?;
    let ix_prefix = SetSolValueCalculatorIxPreAccs(*ix_prefix);

    let list = lst_state_list_checked(abr.get(*ix_prefix.lst_state_list()))?;
    let lst_state = list
        .0
        .get(lst_idx)
        .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidLstIndex))?;
    let lst_mint_acc = abr.get(*ix_prefix.lst_mint());
    let token_prog = lst_mint_acc.owner();

    let expected_reserves =
        create_raw_pool_reserves_addr(token_prog, &lst_state.mint, &lst_state.pool_reserves_bump)
            .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidReserves))?;

    let pool = pool_state_v2_checked(abr.get(*ix_prefix.pool_state()))?;

    let expected_pks = NewSetSolValueCalculatorIxPreAccsBuilder::start()
        .with_admin(&pool.admin)
        .with_lst_mint(&lst_state.mint)
        .with_lst_state_list(&LST_STATE_LIST_ID)
        .with_pool_reserves(&expected_reserves)
        .with_pool_state(&POOL_STATE_ID)
        .build();
    verify_pks(abr, &ix_prefix.0, &expected_pks.0)?;

    verify_signers(abr, &ix_prefix.0, &SET_SOL_VALUE_CALC_IX_PRE_IS_SIGNER.0)?;

    verify_not_rebalancing_and_not_disabled(pool)?;

    let [(calc_prog, calc)] = split_suf_accs(suf, &[])?;
    verify_sol_value_calculator_is_program(abr.get(calc_prog))?;

    Ok(SetSolValueCalculatorIxAccounts {
        ix_prefix,
        calc_prog,
        calc,
    })
}

#[inline]
pub fn process_set_sol_value_calculator(
    abr: &mut Abr,
    cpi: &mut Cpi,
    SetSolValueCalculatorIxAccounts {
        ix_prefix,
        calc_prog,
        calc,
    }: &SetSolValueCalculatorIxAccounts,
    lst_idx: usize,
    clock: &Clock,
) -> Result<(), ProgramError> {
    pool_state_v2_checked_mut(abr.get_mut(*ix_prefix.pool_state()))?
        .release_yield(clock.slot)
        .map_err(Inf1CtlCustomProgErr)?;

    let new_calc_prog = *abr.get(*calc_prog).key();

    let list = lst_state_list_checked_mut(abr.get_mut(*ix_prefix.lst_state_list()))?;
    let lst_state = list
        .0
        .get_mut(lst_idx)
        .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidLstIndex))?;

    lst_state.sol_value_calculator = new_calc_prog;

    lst_ssv_uy(
        abr,
        cpi,
        &SyncSolValueIxAccs {
            ix_prefix: NewSyncSolValueIxPreAccsBuilder::start()
                .with_lst_mint(*ix_prefix.lst_mint())
                .with_pool_state(*ix_prefix.pool_state())
                .with_lst_state_list(*ix_prefix.lst_state_list())
                .with_pool_reserves(*ix_prefix.pool_reserves())
                .build(),
            calc_prog: new_calc_prog,
            calc,
        },
        lst_idx,
    )?;

    Ok(())
}
