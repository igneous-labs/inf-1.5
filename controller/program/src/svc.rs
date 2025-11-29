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
#[inline]
pub fn lst_sync_sol_val<'acc>(
    abr: &mut Abr,
    cpi: &mut Cpi,
    sync_sol_val_accs: SyncSolValIxAccounts<'_, 'acc>,
    lst_index: usize,
) -> Result<(), ProgramError> {
    lst_sync_sol_val_inf_supply_changed(abr, cpi, sync_sol_val_accs, lst_index, SnapU64::memset(1))
}

/// TODO: use return value to create yield update event for self-cpi logging
#[inline]
pub fn lst_sync_sol_val_inf_supply_changed<'acc>(
    abr: &mut Abr,
    cpi: &mut Cpi,
    sync_sol_val_accs: SyncSolValIxAccounts<'_, 'acc>,
    lst_index: usize,
    inf_supply: SnapU64,
) -> Result<(), ProgramError> {
    let SyncSolValueIxAccs {
        ix_prefix,
        calc_prog,
        calc,
    } = sync_sol_val_accs;

    // Sync sol value for input LST
    let lst_balance = get_token_account_amount(abr.get(*ix_prefix.pool_reserves()).data())?;
    let cpi_retval = cpi_lst_to_sol(
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
    )?;
    let lst_new = *cpi_retval.start();

    let list = lst_state_list_checked_mut(abr.get_mut(*ix_prefix.lst_state_list()))?;
    let lst_state = list
        .0
        .get_mut(lst_index)
        .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidLstIndex))?;

    let lst_old = lst_state.sol_value;
    lst_state.sol_value = lst_new;

    let ps = pool_state_v2_checked_mut(abr.get_mut(*ix_prefix.pool_state()))?;
    let old_pool_lamports = PoolSvLamports::snap(ps);
    let mut refs = PoolSvMutRefs::from_pool_state_v2(ps);

    let new_pool_lamports = SyncSolVal {
        lst: NewSnapBuilder::start()
            .with_old(lst_old)
            .with_new(lst_new)
            .build(),
        inf_supply,
    }
    .exec_checked(old_pool_lamports)
    .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::MathError))?;

    refs.update(new_pool_lamports);

    Ok(())
}
