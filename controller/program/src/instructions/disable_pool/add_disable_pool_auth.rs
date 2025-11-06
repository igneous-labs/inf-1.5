use inf1_ctl_jiminy::{
    account_utils::{
        disable_pool_auth_list_checked, disable_pool_auth_list_checked_mut, pool_state_checked,
    },
    err::Inf1CtlErr,
    instructions::disable_pool::add_disable_pool_auth::{
        AddDisablePoolAuthIxAccs, NewAddDisablePoolAuthIxAccsBuilder,
        ADD_DISABLE_POOL_AUTH_IX_IS_SIGNER,
    },
    keys::{DISABLE_POOL_AUTHORITY_LIST_ID, POOL_STATE_ID, SYS_PROG_ID},
    program_err::Inf1CtlCustomProgErr,
};
use jiminy_cpi::{
    account::{Abr, AccountHandle},
    program_error::{ProgramError, NOT_ENOUGH_ACCOUNT_KEYS},
    Cpi,
};
use jiminy_sysvar_rent::{sysvar::SimpleSysvar, Rent};
use sanctum_system_jiminy::sanctum_system_core::instructions::transfer::NewTransferIxAccsBuilder;

use crate::{
    utils::extend_disable_pool_auth_list,
    verify::{verify_disable_pool_auth_list_no_dup, verify_pks, verify_signers},
};

type AddDisablePoolAuthAccounts<'acc> = AddDisablePoolAuthIxAccs<AccountHandle<'acc>>;

#[inline]
pub fn add_disable_pool_auth_accs_checked<'acc>(
    abr: &Abr,
    accs: &[AccountHandle<'acc>],
) -> Result<AddDisablePoolAuthAccounts<'acc>, ProgramError> {
    let accs = accs.first_chunk().ok_or(NOT_ENOUGH_ACCOUNT_KEYS)?;
    let accs = AddDisablePoolAuthIxAccs(*accs);

    let pool = pool_state_checked(abr.get(*accs.pool_state()))?;

    let expected_pks = NewAddDisablePoolAuthIxAccsBuilder::start()
        .with_pool_state(&POOL_STATE_ID)
        .with_disable_pool_auth_list(&DISABLE_POOL_AUTHORITY_LIST_ID)
        .with_system_program(&SYS_PROG_ID)
        .with_admin(&pool.admin)
        // Free: payer can be any signing pubkey with funds
        .with_payer(abr.get(*accs.payer()).key())
        // Free: admin is free to add any pubkey as a new disable pool auth
        .with_new(abr.get(*accs.new()).key())
        .build();
    verify_pks(abr, &accs.0, &expected_pks.0)?;

    verify_signers(abr, &accs.0, &ADD_DISABLE_POOL_AUTH_IX_IS_SIGNER.0)?;

    let disable_pool_auth_list =
        disable_pool_auth_list_checked(abr.get(*accs.disable_pool_auth_list()))?;
    verify_disable_pool_auth_list_no_dup(disable_pool_auth_list.0, abr.get(*accs.new()).key())?;

    Ok(accs)
}

#[inline]
pub fn process_add_disable_pool_auth(
    abr: &mut Abr,
    cpi: &mut Cpi,
    accs: &AddDisablePoolAuthAccounts,
) -> Result<(), ProgramError> {
    extend_disable_pool_auth_list(
        abr,
        cpi,
        &NewTransferIxAccsBuilder::start()
            .with_from(*accs.payer())
            .with_to(*accs.disable_pool_auth_list())
            .build(),
        &Rent::get()?,
    )?;
    let new_auth = *abr.get(*accs.new()).key();
    let list = disable_pool_auth_list_checked_mut(abr.get_mut(*accs.disable_pool_auth_list()))?;
    let new_entry = list.0.last_mut().ok_or(Inf1CtlCustomProgErr(
        Inf1CtlErr::InvalidDisablePoolAuthorityListData,
    ))?;
    *new_entry = new_auth;
    Ok(())
}
