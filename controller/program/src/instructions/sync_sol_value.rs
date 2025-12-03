use inf1_core::instructions::sync_sol_value::SyncSolValueIxAccs;
use inf1_ctl_jiminy::{
    account_utils::{lst_state_list_checked, lst_state_list_get, pool_state_v2_checked_mut},
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
use jiminy_sysvar_clock::Clock;

use crate::{
    acc_migrations::pool_state,
    svc::lst_sync_sol_val,
    utils::split_suf_accs,
    verify::{verify_not_rebalancing_and_not_disabled_v2, verify_pks},
    Cpi,
};

#[inline]
pub fn process_sync_sol_value(
    abr: &mut Abr,
    cpi: &mut Cpi,
    accounts: &[AccountHandle<'_>],
    lst_idx: usize,
    clock: &Clock,
) -> Result<(), ProgramError> {
    let (ix_prefix, suf) = accounts
        .split_first_chunk()
        .ok_or(NOT_ENOUGH_ACCOUNT_KEYS)?;
    let ix_prefix = SyncSolValueIxPreAccs(*ix_prefix);

    pool_state::v2::migrate_idmpt(abr.get_mut(*ix_prefix.pool_state()), clock)?;

    let list = lst_state_list_checked(abr.get(*ix_prefix.lst_state_list()))?;
    let lst_state = lst_state_list_get(list, lst_idx)?;
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

    let [(calc_prog, calc)] = split_suf_accs(suf, &[])?;
    verify_pks(abr, &[calc_prog], &[&lst_state.sol_value_calculator])?;

    let pool = pool_state_v2_checked_mut(abr.get_mut(*ix_prefix.pool_state()))?;
    verify_not_rebalancing_and_not_disabled_v2(pool)?;

    pool.release_yield(clock.slot)
        .map_err(Inf1CtlCustomProgErr)?;

    lst_sync_sol_val(
        abr,
        cpi,
        &SyncSolValueIxAccs {
            ix_prefix,
            calc_prog: *abr.get(calc_prog).key(),
            calc,
        },
        lst_idx,
    )?;

    Ok(())
}
