use inf1_core::{instructions::sync_sol_value::SyncSolValueIxAccs, sync::SyncSolVal};
use inf1_ctl_jiminy::{
    account_utils::{lst_state_list_checked_mut, pool_state_v2_checked_mut},
    cpi::SyncSolValueIxPreAccountHandles,
    err::Inf1CtlErr,
    program_err::Inf1CtlCustomProgErr,
    typedefs::{
        pool_sv::{PoolSvLamports, PoolSvMutRefs},
        snap::{NewSnapBuilder, SnapU64},
    },
};

use inf1_svc_jiminy::cpi::cpi_lst_to_sol;
pub use inf1_svc_jiminy::{
    cpi::IxAccountHandles as SvcIxAccountHandles,
    instructions::NewIxPreAccsBuilder as NewSvcIxPreAccsBuilder,
};
use jiminy_cpi::{
    account::{Abr, AccountHandle},
    program_error::ProgramError,
};

use crate::{token::get_token_account_amount, Cpi};

pub type SyncSolValIxAccounts<'a, 'acc> = SyncSolValueIxAccs<
    AccountHandle<'acc>,
    SyncSolValueIxPreAccountHandles<'acc>,
    &'a [AccountHandle<'acc>],
>;

/// TODO: use return value to create yield update event for self-cpi logging
/// TODO: need variant with inf_supply snap for add/remove liquidity
/// TODO: need variant without UpdateYield for the last sync in StartRebalance
#[inline]
pub fn lst_sync_sol_val<'acc>(
    abr: &mut Abr,
    cpi: &mut Cpi,
    sync_sol_val_accs: SyncSolValIxAccounts<'_, 'acc>,
    lst_index: usize,
) -> Result<(), ProgramError> {
    let lst_new = cpi_lst_reserves_sol_val(abr, cpi, sync_sol_val_accs)?;
    let lst_snap = update_lst_state(
        abr,
        *sync_sol_val_accs.ix_prefix.lst_state_list(),
        lst_index,
        lst_new,
    )?;
    update_pool_state(abr, *sync_sol_val_accs.ix_prefix.pool_state(), lst_snap)
}

fn update_pool_state(
    abr: &mut Abr,
    pool_state: AccountHandle,
    lst: SnapU64,
) -> Result<(), ProgramError> {
    let ps = pool_state_v2_checked_mut(abr.get_mut(pool_state))?;
    let old = PoolSvLamports::from_pool_state_v2(ps);
    let new = SyncSolVal { lst }
        .exec(old)
        .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::MathError))?;
    PoolSvMutRefs::from_pool_state_v2(ps).update(new);
    Ok(())
}

#[inline]
fn cpi_lst_reserves_sol_val<'acc>(
    abr: &mut Abr,
    cpi: &mut Cpi,
    sync_sol_val_accs: SyncSolValIxAccounts<'_, 'acc>,
) -> Result<u64, ProgramError> {
    let SyncSolValueIxAccs {
        ix_prefix,
        calc_prog,
        calc,
    } = sync_sol_val_accs;
    let lst_balance = get_token_account_amount(abr.get(*ix_prefix.pool_reserves()).data())?;
    Ok(*cpi_lst_to_sol(
        cpi,
        abr,
        calc_prog,
        lst_balance,
        SvcIxAccountHandles::new(
            NewSvcIxPreAccsBuilder::start()
                .with_lst_mint(*ix_prefix.lst_mint())
                .build(),
            calc,
        ),
    )?
    .start())
}

/// Returns change in SOL value of LST
fn update_lst_state(
    abr: &mut Abr,
    lst_state_list: AccountHandle,
    lst_index: usize,
    new_sol_val: u64,
) -> Result<SnapU64, ProgramError> {
    let list = lst_state_list_checked_mut(abr.get_mut(lst_state_list))?;
    let lst_state = list
        .0
        .get_mut(lst_index)
        .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidLstIndex))?;
    let lst_old = lst_state.sol_value;
    lst_state.sol_value = new_sol_val;
    Ok(NewSnapBuilder::start()
        .with_old(lst_old)
        .with_new(new_sol_val)
        .build())
}
