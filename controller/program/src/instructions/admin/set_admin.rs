use inf1_ctl_jiminy::{
    account_utils::{pool_state_v2_checked, pool_state_v2_checked_mut},
    instructions::admin::set_admin::{
        NewSetAdminIxAccsBuilder, SetAdminIxAccs, SET_ADMIN_IX_IS_SIGNER,
    },
    keys::POOL_STATE_ID,
};
use jiminy_cpi::{
    account::{Abr, AccountHandle},
    program_error::{ProgramError, NOT_ENOUGH_ACCOUNT_KEYS},
};
use jiminy_sysvar_clock::Clock;

use crate::{
    acc_migrations::pool_state,
    verify::{verify_pks, verify_signers},
};

type SetAdminIxAccounts<'acc> = SetAdminIxAccs<AccountHandle<'acc>>;

#[inline]
pub fn set_admin_accs_checked<'acc>(
    abr: &mut Abr,
    accs: &[AccountHandle<'acc>],
    clock: &Clock,
) -> Result<SetAdminIxAccounts<'acc>, ProgramError> {
    let accs = accs.first_chunk().ok_or(NOT_ENOUGH_ACCOUNT_KEYS)?;
    let accs = SetAdminIxAccs(*accs);

    pool_state::v2::migrate_idmpt(abr.get_mut(*accs.pool_state()), clock)?;

    let pool = pool_state_v2_checked(abr.get(*accs.pool_state()))?;

    let expected_pks = NewSetAdminIxAccsBuilder::start()
        .with_pool_state(&POOL_STATE_ID)
        .with_curr(&pool.admin)
        // Free: current admin is free to set new admin to whatever pk as pleased
        .with_new(abr.get(*accs.new()).key())
        .build();
    verify_pks(abr, &accs.0, &expected_pks.0)?;

    verify_signers(abr, &accs.0, &SET_ADMIN_IX_IS_SIGNER.0)?;

    Ok(accs)
}

#[inline]
pub fn process_set_admin(abr: &mut Abr, accs: SetAdminIxAccounts) -> Result<(), ProgramError> {
    let new_admin = *abr.get(*accs.new()).key();
    let pool = pool_state_v2_checked_mut(abr.get_mut(*accs.pool_state()))?;
    pool.admin = new_admin;
    Ok(())
}
