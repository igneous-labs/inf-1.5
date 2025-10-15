use inf1_ctl_jiminy::{
    accounts::{
        lst_state_list::{LstStatePackedList, LstStatePackedListMut},
        pool_state::PoolState,
    },
    cpi::SetSolValueCalculatorIxPreAccountHandles,
    err::Inf1CtlErr,
    instructions::set_sol_value_calculator::{
        NewSetSolValueCalculatorIxPreAccsBuilder, SetSolValueCalculatorIxPreAccs,
    },
    keys::{LST_STATE_LIST_ID, POOL_STATE_ID},
    pda_onchain::create_raw_pool_reserves_addr,
    program_err::Inf1CtlCustomProgErr,
};
use inf1_svc_jiminy::cpi::cpi_lst_to_sol;
use jiminy_cpi::{
    account::AccountHandle,
    program_error::{ProgramError, INVALID_ACCOUNT_DATA, NOT_ENOUGH_ACCOUNT_KEYS},
};
use sanctum_spl_token_jiminy::sanctum_spl_token_core::state::account::{
    RawTokenAccount, TokenAccount,
};
use std::ops::Range;

use inf1_core::instructions::set_sol_value_calculator::SetSolValueCalculatorIxAccs;

use crate::{
    instructions::sync_sol_value::sync_sol_val_with_retval,
    svc::NewSvcIxPreAccsBuilder,
    verify::{verify_not_rebalancing_and_not_disabled, verify_pks},
    Accounts, Cpi,
};

pub type SetSolValueCalculatorIxAccounts<'acc> = SetSolValueCalculatorIxAccs<
    AccountHandle<'acc>,
    SetSolValueCalculatorIxPreAccountHandles<'acc>,
    Range<usize>,
>;

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
        .with_pool_state(&POOL_STATE_ID)
        .with_pool_reserves(&expected_reserves)
        .build();
    verify_pks(accounts, &ix_prefix.0, &expected_pks.0)?;

    let calc_prog = suf.first().ok_or(NOT_ENOUGH_ACCOUNT_KEYS)?;
    verify_pks(accounts, &[*calc_prog], &[&lst_state.sol_value_calculator])?;

    verify_not_rebalancing_and_not_disabled(pool)?;

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

    let lst_balance = RawTokenAccount::of_acc_data(accounts.get(*ix_prefix.pool_reserves()).data())
        .and_then(TokenAccount::try_from_raw)
        .map(|a| a.amount())
        .ok_or(INVALID_ACCOUNT_DATA)?;
    let retval = cpi_lst_to_sol(
        cpi,
        accounts,
        calc_prog,
        lst_balance,
        NewSvcIxPreAccsBuilder::start()
            .with_lst_mint(*ix_prefix.lst_mint())
            .build(),
        calc,
    )?;
    sync_sol_val_with_retval(
        accounts,
        *ix_prefix.pool_state(),
        *ix_prefix.lst_state_list(),
        lst_idx,
        retval,
    )
}
