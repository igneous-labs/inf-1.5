use inf1_ctl_jiminy::{
    accounts::{lst_state_list::LstStatePackedList, pool_state::PoolState},
    err::Inf1CtlErr,
    instructions::sync_sol_value::{NewSyncSolValueIxPreAccsBuilder, SyncSolValueIxPreAccs},
    keys::{LST_STATE_LIST_ID, POOL_STATE_ID},
    pda_onchain::create_raw_pool_reserves_addr,
    program_err::Inf1CtlCustomProgErr,
};
use jiminy_cpi::program_error::{ProgramError, NOT_ENOUGH_ACCOUNT_KEYS};

use crate::{
    svc::lst_sync_sol_val_unchecked,
    verify::{verify_not_rebalancing_and_not_disabled, verify_pks},
    Accounts, Cpi,
};

#[inline]
pub fn process_sync_sol_value(
    accounts: &mut Accounts<'_>,
    lst_idx: usize,
    cpi: &mut Cpi,
) -> Result<(), ProgramError> {
    let (ix_prefix, suf) = accounts
        .as_slice()
        .split_first_chunk()
        .ok_or(NOT_ENOUGH_ACCOUNT_KEYS)?;
    let ix_prefix = SyncSolValueIxPreAccs(*ix_prefix);
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

    let expected_pks = NewSyncSolValueIxPreAccsBuilder::start()
        .with_lst_mint(&lst_state.mint)
        .with_lst_state_list(&LST_STATE_LIST_ID)
        .with_pool_state(&POOL_STATE_ID)
        .with_pool_reserves(&expected_reserves)
        .build();
    verify_pks(accounts, &ix_prefix.0, &expected_pks.0)?;

    let calc_prog = suf.first().ok_or(NOT_ENOUGH_ACCOUNT_KEYS)?;
    verify_pks(accounts, &[*calc_prog], &[&lst_state.sol_value_calculator])?;

    // safety: account data is 8-byte aligned
    let pool = unsafe { PoolState::of_acc_data(accounts.get(*ix_prefix.pool_state()).data()) }
        .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidPoolStateData))?;
    verify_not_rebalancing_and_not_disabled(pool)?;

    let calc = ix_prefix.0.len() + 1..accounts.as_slice().len();

    lst_sync_sol_val_unchecked(accounts, cpi, ix_prefix, lst_idx, *calc_prog, calc)
}
