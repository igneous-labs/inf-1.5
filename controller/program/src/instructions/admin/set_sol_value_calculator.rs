use inf1_ctl_jiminy::{
    accounts::{
        lst_state_list::{LstStatePackedList, LstStatePackedListMut},
        pool_state::PoolState,
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
    account::AccountHandle,
    program_error::{ProgramError, NOT_ENOUGH_ACCOUNT_KEYS},
};
use std::ops::Range;

use inf1_core::instructions::admin::set_sol_value_calculator::SetSolValueCalculatorIxAccs;
use inf1_core::instructions::sync_sol_value::SyncSolValueIxAccs;

use crate::{
    svc::lst_sync_sol_val_unchecked,
    verify::{
        log_and_return_acc_privilege_err, verify_not_rebalancing_and_not_disabled, verify_pks,
        verify_signers, verify_sol_value_calculator_is_program,
    },
    Accounts, Cpi,
};

pub type SetSolValueCalculatorIxAccounts<'acc> = SetSolValueCalculatorIxAccs<
    AccountHandle<'acc>,
    SetSolValueCalculatorIxPreAccountHandles<'acc>,
    Range<usize>,
>;

/// Returns (prefix, sol_val_calc_program, remaining accounts)
#[inline]
fn set_sol_value_calculator_accs_checked<'acc>(
    accounts: &Accounts<'acc>,
    lst_idx: usize,
) -> Result<SetSolValueCalculatorIxAccounts<'acc>, ProgramError> {
    let (ix_prefix, suf) = accounts
        .as_slice()
        .split_first_chunk()
        .ok_or(NOT_ENOUGH_ACCOUNT_KEYS)?;
    let ix_prefix = SetSolValueCalculatorIxPreAccs(*ix_prefix);
    let list = LstStatePackedList::of_acc_data(accounts.get(*ix_prefix.lst_state_list()).data())
        .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidLstStateListData))?;
    let lst_state = list
        .0
        .get(lst_idx)
        .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidLstIndex))?;
    let lst_mint_acc = accounts.get(*ix_prefix.lst_mint());
    let token_prog = lst_mint_acc.owner();
    // safety: account data is 8-byte aligned
    let lst_state = unsafe { lst_state.as_lst_state() };
    let expected_reserves =
        create_raw_pool_reserves_addr(token_prog, &lst_state.mint, &lst_state.pool_reserves_bump)
            .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidReserves))?;

    // safety: account data is 8-byte aligned
    let pool = unsafe { PoolState::of_acc_data(accounts.get(*ix_prefix.pool_state()).data()) }
        .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidPoolStateData))?;

    let expected_pks = NewSetSolValueCalculatorIxPreAccsBuilder::start()
        .with_admin(&pool.admin)
        .with_lst_mint(&lst_state.mint)
        .with_lst_state_list(&LST_STATE_LIST_ID)
        .with_pool_reserves(&expected_reserves)
        .with_pool_state(&POOL_STATE_ID)
        .build();
    verify_pks(accounts, &ix_prefix.0, &expected_pks.0)?;

    verify_signers(
        accounts,
        &ix_prefix.0,
        &SET_SOL_VALUE_CALC_IX_PRE_IS_SIGNER.0,
    )
    .map_err(|expected_signer| log_and_return_acc_privilege_err(accounts, *expected_signer))?;

    verify_not_rebalancing_and_not_disabled(pool)?;

    let calc_prog = suf.first().ok_or(NOT_ENOUGH_ACCOUNT_KEYS)?;
    verify_sol_value_calculator_is_program(accounts.get(*calc_prog))?;

    Ok(SetSolValueCalculatorIxAccounts {
        ix_prefix,
        calc_prog: *calc_prog,
        calc: ix_prefix.0.len() + 1..accounts.as_slice().len(),
    })
}

#[inline]
pub fn process_set_sol_value_calculator(
    accounts: &mut Accounts<'_>,
    lst_idx: usize,
    cpi: &mut Cpi,
) -> Result<(), ProgramError> {
    let SetSolValueCalculatorIxAccounts {
        ix_prefix,
        calc_prog,
        calc,
    } = set_sol_value_calculator_accs_checked(accounts, lst_idx)?;

    let calc_key = *accounts.get(calc_prog).key();

    let list = LstStatePackedListMut::of_acc_data(
        accounts.get_mut(*ix_prefix.lst_state_list()).data_mut(),
    )
    .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidLstStateListData))?;
    let lst_state = list
        .0
        .get_mut(lst_idx)
        .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidLstIndex))?;
    // safety: account data is 8-byte aligned
    let lst_state = unsafe { lst_state.as_lst_state_mut() };

    lst_state.sol_value_calculator = calc_key;

    lst_sync_sol_val_unchecked(
        accounts,
        cpi,
        SyncSolValueIxAccs {
            ix_prefix: NewSyncSolValueIxPreAccsBuilder::start()
                .with_lst_mint(*ix_prefix.lst_mint())
                .with_pool_state(*ix_prefix.pool_state())
                .with_lst_state_list(*ix_prefix.lst_state_list())
                .with_pool_reserves(*ix_prefix.pool_reserves())
                .build(),
            calc_prog,
            calc,
        },
        lst_idx,
    )
}
