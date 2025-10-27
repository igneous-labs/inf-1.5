use inf1_core::{instructions::sync_sol_value::SyncSolValueIxAccs, sync::SyncSolVal};
use inf1_ctl_jiminy::{
    accounts::{lst_state_list::LstStatePackedListMut, pool_state::PoolState},
    cpi::SyncSolValueIxPreAccountHandles,
    err::Inf1CtlErr,
    program_err::Inf1CtlCustomProgErr,
};
use inf1_svc_jiminy::{
    cpi::{cpi_lst_to_sol, IxAccountHandles as SvcIxAccountHandles},
    instructions::NewIxPreAccsBuilder as NewSvcIxPreAccsBuilder,
};
use jiminy_cpi::{
    account::{Abr, AccountHandle},
    program_error::{ProgramError, INVALID_ACCOUNT_DATA},
};
use sanctum_spl_token_jiminy::sanctum_spl_token_core::state::account::{
    RawTokenAccount, TokenAccount,
};

use crate::Cpi;

pub type SyncSolValIxAccounts<'a, 'acc> = SyncSolValueIxAccs<
    AccountHandle<'acc>,
    SyncSolValueIxPreAccountHandles<'acc>,
    &'a [AccountHandle<'acc>],
>;

#[inline]
pub fn lst_sync_sol_val_unchecked<'acc>(
    abr: &mut Abr,
    cpi: &mut Cpi,
    sync_sol_val_accs: SyncSolValIxAccounts<'_, 'acc>,
    lst_index: usize,
) -> Result<(), ProgramError> {
    let SyncSolValueIxAccs {
        ix_prefix,
        calc_prog,
        calc,
    } = sync_sol_val_accs;

    let pool_reserves = *ix_prefix.pool_reserves();
    let lst_state_list = *ix_prefix.lst_state_list();
    let pool_state = *ix_prefix.pool_state();
    let lst_mint = *ix_prefix.lst_mint();

    // Sync sol value for input LST
    let lst_balance = RawTokenAccount::of_acc_data(abr.get(pool_reserves).data())
        .and_then(TokenAccount::try_from_raw)
        .map(|a| a.amount())
        .ok_or(INVALID_ACCOUNT_DATA)?;

    let cpi_retval = cpi_lst_to_sol(
        cpi,
        abr,
        calc_prog,
        lst_balance,
        SvcIxAccountHandles::new(
            NewSvcIxPreAccsBuilder::start()
                .with_lst_mint(lst_mint)
                .build(),
            calc,
        ),
    )?;

    let lst_new = *cpi_retval.start();

    let list = LstStatePackedListMut::of_acc_data(abr.get_mut(lst_state_list).data_mut())
        .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidLstStateListData))?;
    let lst_state = list
        .0
        .get_mut(lst_index)
        .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidLstIndex))?;
    // safety: account data is 8-byte aligned
    let lst_state = unsafe { lst_state.as_lst_state_mut() };
    let lst_old = lst_state.sol_value;
    lst_state.sol_value = lst_new;

    // safety: account data is 8-byte aligned
    let pool = unsafe { PoolState::of_acc_data_mut(abr.get_mut(pool_state).data_mut()) }
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
