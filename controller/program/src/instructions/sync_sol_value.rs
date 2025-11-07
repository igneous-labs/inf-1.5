use inf1_core::instructions::sync_sol_value::SyncSolValueIxAccs;
use inf1_ctl_jiminy::{
    account_utils::{lst_state_list_checked, pool_state_checked},
    err::Inf1CtlErr,
    instructions::sync_sol_value::{NewSyncSolValueIxPreAccsBuilder, SyncSolValueIxPreAccs},
    keys::{LST_STATE_LIST_ID, POOL_STATE_ID},
    pda_onchain::create_raw_pool_reserves_addr,
    program_err::Inf1CtlCustomProgErr,
};
use jiminy_cpi::{
    account::{Abr, AccountHandle},
    program_error::{ProgramError, NOT_ENOUGH_ACCOUNT_KEYS},
};

use crate::{
    svc::lst_sync_sol_val_unchecked,
    verify::{verify_not_rebalancing_and_not_disabled, verify_pks},
    Cpi,
};

#[inline]
pub fn process_sync_sol_value(
    abr: &mut Abr,
    accounts: &[AccountHandle<'_>],
    lst_idx: usize,
    cpi: &mut Cpi,
) -> Result<(), ProgramError> {
    let (ix_prefix, suf) = accounts
        .split_first_chunk()
        .ok_or(NOT_ENOUGH_ACCOUNT_KEYS)?;
    let ix_prefix = SyncSolValueIxPreAccs(*ix_prefix);
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

    let expected_pks = NewSyncSolValueIxPreAccsBuilder::start()
        .with_lst_mint(&lst_state.mint)
        .with_lst_state_list(&LST_STATE_LIST_ID)
        .with_pool_state(&POOL_STATE_ID)
        .with_pool_reserves(&expected_reserves)
        .build();
    verify_pks(abr, &ix_prefix.0, &expected_pks.0)?;

    let (calc_prog, calc) = suf.split_first().ok_or(NOT_ENOUGH_ACCOUNT_KEYS)?;
    verify_pks(abr, &[*calc_prog], &[&lst_state.sol_value_calculator])?;

    let pool = pool_state_checked(abr.get(*ix_prefix.pool_state()))?;
    verify_not_rebalancing_and_not_disabled(pool)?;

    lst_sync_sol_val_unchecked(
        abr,
        cpi,
        SyncSolValueIxAccs {
            ix_prefix,
            calc_prog: *calc_prog,
            calc,
        },
        lst_idx,
    )
}
