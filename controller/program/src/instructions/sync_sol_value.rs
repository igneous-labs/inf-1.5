use std::ops::RangeInclusive;

use inf1_core::{instructions::sync_sol_value::SyncSolValueIxAccs, sync::SyncSolVal};
use inf1_ctl_jiminy::{
    accounts::{
        lst_state_list::{LstStatePackedList, LstStatePackedListMut},
        pool_state::PoolState,
    },
    cpi::SyncSolValueIxPreAccountHandles,
    err::Inf1CtlErr,
    instructions::sync_sol_value::{NewSyncSolValueIxPreAccsBuilder, SyncSolValueIxPreAccs},
    keys::{LST_STATE_LIST_ID, POOL_STATE_ID},
    pda_onchain::create_raw_pool_reserves_addr,
    program_err::Inf1CtlCustomProgErr,
};
use inf1_svc_jiminy::cpi::prep_cpi_lst_to_sol;
use jiminy_cpi::{
    account::AccountHandle,
    program_error::{ProgramError, INVALID_ACCOUNT_DATA, NOT_ENOUGH_ACCOUNT_KEYS},
};
use sanctum_spl_token_jiminy::sanctum_spl_token_core::state::account::{
    RawTokenAccount, TokenAccount,
};

use crate::{
    svc::{NewSvcIxPreAccsBuilder, SvcIxAccountHandles},
    verify::{
        verify_not_rebalancing_and_not_disabled, verify_pks, verify_sol_val_calc_prog,
        wrong_acc_logmapper,
    },
    Accounts, Cpi,
};

pub type SyncSolValIxAccounts<'a, 'acc> = SyncSolValueIxAccs<
    AccountHandle<'acc>,
    SyncSolValueIxPreAccountHandles<'acc>,
    &'a [AccountHandle<'acc>],
>;

/// Returns (prefix, sol_val_calc_program, remaining accounts)
#[inline]
fn sync_sol_value_accs_checked<'a, 'acc>(
    accounts: &'a Accounts<'acc>,
    lst_idx: usize,
) -> Result<SyncSolValIxAccounts<'a, 'acc>, ProgramError> {
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

    verify_pks(accounts, &ix_prefix.0, &expected_pks.0).map_err(wrong_acc_logmapper(accounts))?;

    let (calc_prog, suf) = suf.split_first().ok_or(NOT_ENOUGH_ACCOUNT_KEYS)?;
    verify_sol_val_calc_prog(accounts, lst_state, *calc_prog)?;

    // safety: account data is 8-byte aligned
    let pool = unsafe { PoolState::of_acc_data(accounts.get(*ix_prefix.pool_state()).data()) }
        .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidPoolStateData))?;

    verify_not_rebalancing_and_not_disabled(pool)?;

    Ok(SyncSolValIxAccounts {
        ix_prefix,
        calc_prog: *calc_prog,
        calc: suf,
    })
}

#[inline]
pub fn process_sync_sol_value(
    accounts: &mut Accounts<'_>,
    lst_idx: usize,
    cpi: &mut Cpi,
) -> Result<(), ProgramError> {
    let SyncSolValIxAccounts {
        ix_prefix,
        calc_prog,
        calc,
    } = sync_sol_value_accs_checked(accounts, lst_idx)?;
    let lst_balance = RawTokenAccount::of_acc_data(accounts.get(*ix_prefix.pool_reserves()).data())
        .and_then(TokenAccount::try_from_raw)
        .map(|a| a.amount())
        .ok_or(INVALID_ACCOUNT_DATA)?;
    // safety: prepped is immediately invoked
    let retval = unsafe {
        prep_cpi_lst_to_sol(
            cpi,
            accounts,
            SvcIxAccountHandles {
                ix_prefix: NewSvcIxPreAccsBuilder::start()
                    .with_lst_mint(*ix_prefix.lst_mint())
                    .build(),
                suf: calc,
            },
            accounts.get(calc_prog).key(),
            lst_balance,
        )?
    }
    .invoke(accounts)?;

    sync_sol_val_with_retval(
        accounts,
        *ix_prefix.pool_state(),
        *ix_prefix.lst_state_list(),
        lst_idx,
        retval,
    )
}

#[inline]
pub fn sync_sol_val_with_retval<'acc>(
    accounts: &mut Accounts<'acc>,
    pool: AccountHandle<'acc>,
    lst_state_list: AccountHandle<'acc>,
    lst_idx: usize,
    // should be value returned by sol val calc program
    retval: RangeInclusive<u64>,
) -> Result<(), ProgramError> {
    let lst_new = *retval.start();

    let list = LstStatePackedListMut::of_acc_data(accounts.get_mut(lst_state_list).data_mut())
        .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidLstStateListData))?;
    let lst_state = list
        .0
        .get_mut(lst_idx)
        .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidLstIndex))?;
    // safety: account data is 8-byte aligned
    let lst_state = unsafe { lst_state.as_lst_state_mut() };
    let lst_old = lst_state.sol_value;
    lst_state.sol_value = lst_new;

    // safety: account data is 8-byte aligned
    let pool = unsafe { PoolState::of_acc_data_mut(accounts.get_mut(pool).data_mut()) }
        .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidPoolStateData))?;
    pool.total_sol_value = SyncSolVal {
        pool_total: pool.total_sol_value,
        lst_old,
        lst_new,
    }
    .exec_checked()
    .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::MathError))?;

    Ok(())
}
