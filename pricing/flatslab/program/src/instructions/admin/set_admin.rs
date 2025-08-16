use inf1_pp_flatslab_core::{
    accounts::{Slab, SlabMut},
    instructions::admin::set_admin::{
        NewSetAdminIxAccsBuilder, SetAdminIxAccs, SetAdminIxKeys, SET_ADMIN_IX_IS_SIGNER,
    },
    keys::SLAB_ID,
};
use jiminy_cpi::{
    account::AccountHandle,
    program_error::{ProgramError, INVALID_ACCOUNT_DATA, NOT_ENOUGH_ACCOUNT_KEYS},
};

use crate::{
    admin_ix_verify_pks_err, admin_ix_verify_signers_err, verify_pks, verify_signers, Accounts,
};

pub type SetAdminIxAccHandles<'a> = SetAdminIxAccs<AccountHandle<'a>>;

fn expected_set_admin_ix_keys<'a>(slab: &'a Slab, new_admin: &'a [u8; 32]) -> SetAdminIxKeys<'a> {
    NewSetAdminIxAccsBuilder::start()
        .with_slab(&SLAB_ID)
        .with_new_admin(new_admin)
        .with_current_admin(slab.admin())
        .build()
}

pub fn set_admin_accs_checked<'acc>(
    accounts: &Accounts<'acc>,
) -> Result<SetAdminIxAccHandles<'acc>, ProgramError> {
    let Some(accs) = accounts.as_slice().first_chunk() else {
        return Err(NOT_ENOUGH_ACCOUNT_KEYS.into());
    };
    let accs = SetAdminIxAccHandles::new(*accs);

    let slab = Slab::of_acc_data(accounts.get(*accs.slab()).data()).ok_or(INVALID_ACCOUNT_DATA)?;

    verify_pks(
        accounts,
        &accs.0,
        &expected_set_admin_ix_keys(&slab, accounts.get(*accs.new_admin()).key()).0,
    )
    .map_err(|(_actual, expected)| admin_ix_verify_pks_err(expected, slab))?;

    verify_signers(accounts, &accs.0, &SET_ADMIN_IX_IS_SIGNER.0)
        .map_err(|expected_signer| admin_ix_verify_signers_err(accounts, *expected_signer, slab))?;

    Ok(accs)
}

pub fn process_set_admin<'acc>(
    accounts: &mut Accounts<'acc>,
    accs: SetAdminIxAccHandles<'acc>,
) -> Result<(), ProgramError> {
    let new_admin_pk = *accounts.get(*accs.new_admin()).key();
    let mut slab = SlabMut::of_acc_data(accounts.get_mut(*accs.slab()).data_mut())
        .ok_or(INVALID_ACCOUNT_DATA)?;
    let (admin, _) = slab.as_mut();
    *admin = new_admin_pk;
    Ok(())
}
