use inf1_core::instructions::sync_sol_value::SyncSolValueIxAccs;
use inf1_ctl_jiminy::{
    account_utils::{
        lst_state_list_checked_mut, lst_state_list_get_mut, pool_state_v2_checked_mut,
    },
    cpi::SyncSolValueIxPreAccountHandles,
    err::Inf1CtlErr,
    program_err::Inf1CtlCustomProgErr,
    sync_sol_val::SyncSolVal,
    typedefs::snap::{NewSnapBuilder, SnapU64},
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

pub type SyncSolValIxAccounts<'a, 'acc> =
    SyncSolValueIxAccs<[u8; 32], SyncSolValueIxPreAccountHandles<'acc>, &'a [AccountHandle<'acc>]>;

/// TODO: use return value to create yield update event for self-cpi logging
/// TODO: need variant without UpdateYield for the last sync in StartRebalance
#[inline]
pub fn lst_sync_sol_val(
    abr: &mut Abr,
    cpi: &mut Cpi,
    sync_sol_val_accs: &SyncSolValIxAccounts,
    lst_index: usize,
) -> Result<(), ProgramError> {
    let lst_new = cpi_lst_reserves_sol_val(abr, cpi, sync_sol_val_accs)?;
    let lst_sol_val = update_lst_state_sol_val(
        abr,
        *sync_sol_val_accs.ix_prefix.lst_state_list(),
        lst_index,
        lst_new,
    )?;
    let ps = pool_state_v2_checked_mut(abr.get_mut(*sync_sol_val_accs.ix_prefix.pool_state()))?;
    ps.apply_ssv_uy(&SyncSolVal { lst_sol_val })
        .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::MathError))?;
    Ok(())
}

#[inline]
pub fn cpi_lst_reserves_sol_val(
    abr: &mut Abr,
    cpi: &mut Cpi,
    sync_sol_val_accs: &SyncSolValIxAccounts,
) -> Result<u64, ProgramError> {
    let SyncSolValueIxAccs {
        ix_prefix,
        calc_prog,
        calc,
    } = sync_sol_val_accs;
    let lst_balance = get_token_account_amount(abr.get(*ix_prefix.pool_reserves()))?;
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
pub fn update_lst_state_sol_val(
    abr: &mut Abr,
    lst_state_list: AccountHandle,
    lst_index: usize,
    new_sol_val: u64,
) -> Result<SnapU64, ProgramError> {
    let list = lst_state_list_checked_mut(abr.get_mut(lst_state_list))?;
    let lst_state = lst_state_list_get_mut(list, lst_index)?;
    let old_sol_val = lst_state.sol_value;
    lst_state.sol_value = new_sol_val;
    Ok(NewSnapBuilder::start()
        .with_old(old_sol_val)
        .with_new(new_sol_val)
        .build())
}
