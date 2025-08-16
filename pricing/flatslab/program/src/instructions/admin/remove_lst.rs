use inf1_pp_flatslab_core::{
    accounts::Slab,
    instructions::admin::remove_lst::{
        NewRemoveLstIxAccsBuilder, RemoveLstIxAccs, REMOVE_LST_IX_IS_SIGNER,
    },
    keys::SLAB_ID,
    typedefs::SlabEntryPacked,
};
use jiminy_cpi::{
    account::AccountHandle,
    program_error::{ProgramError, INVALID_ACCOUNT_DATA, NOT_ENOUGH_ACCOUNT_KEYS},
};
use jiminy_sysvar_rent::{sysvar::SimpleSysvar, Rent};

use crate::{
    admin_ix_verify_pks_err, admin_ix_verify_signers_err, verify_pks, verify_signers, Accounts,
};

pub type RemoveLstIxAccHandles<'a> = RemoveLstIxAccs<AccountHandle<'a>>;

pub fn remove_lst_accs_checked<'acc>(
    accounts: &Accounts<'acc>,
) -> Result<RemoveLstIxAccHandles<'acc>, ProgramError> {
    let Some(accs) = accounts.as_slice().first_chunk() else {
        return Err(NOT_ENOUGH_ACCOUNT_KEYS.into());
    };
    let accs = RemoveLstIxAccHandles::new(*accs);

    let slab = Slab::of_acc_data(accounts.get(*accs.slab()).data()).ok_or(INVALID_ACCOUNT_DATA)?;

    let expected_keys = NewRemoveLstIxAccsBuilder::start()
        .with_slab(&SLAB_ID)
        .with_admin(slab.admin())
        .with_mint(accounts.get(*accs.mint()).key())
        .with_refund_rent_to(accounts.get(*accs.refund_rent_to()).key())
        .build();

    verify_pks(accounts, &accs.0, &expected_keys.0)
        .map_err(|(_actual, expected)| admin_ix_verify_pks_err(expected, slab))?;

    verify_signers(accounts, &accs.0, &REMOVE_LST_IX_IS_SIGNER.0)
        .map_err(|expected_signer| admin_ix_verify_signers_err(accounts, *expected_signer, slab))?;

    Ok(accs)
}

pub fn process_remove_lst<'acc>(
    accounts: &mut Accounts<'acc>,
    accs: RemoveLstIxAccHandles<'acc>,
) -> Result<(), ProgramError> {
    let mint = *accounts.get(*accs.mint()).key();
    let slab_acc = accounts.get_mut(*accs.slab());
    let slab = Slab::of_acc_data(slab_acc.data()).ok_or(INVALID_ACCOUNT_DATA)?;
    let idx = match slab.entries().find_idx_by_mint(&mint) {
        Ok(i) => i,
        // mint already doesnt exist
        Err(_) => return Ok(()),
    };

    // shrink acc
    let old_acc_len = slab_acc.data_len();
    let byte_offset = Slab::entry_byte_offset(idx);
    slab_acc.data_mut().copy_within(
        (byte_offset + size_of::<SlabEntryPacked>())..old_acc_len,
        byte_offset,
    );
    slab_acc.shrink_by(size_of::<SlabEntryPacked>())?;
    let new_acc_len = slab_acc.data_len();

    let lamports_surplus = slab_acc
        .lamports()
        .checked_sub(Rent::get()?.min_balance(new_acc_len))
        .ok_or(INVALID_ACCOUNT_DATA)?;
    if lamports_surplus > 0 {
        accounts.transfer_direct(*accs.slab(), *accs.refund_rent_to(), lamports_surplus)?;
    }

    Ok(())
}
