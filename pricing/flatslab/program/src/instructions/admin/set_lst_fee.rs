use inf1_pp_flatslab_core::{
    accounts::{Slab, SlabMut},
    instructions::admin::set_lst_fee::{
        NewSetLstFeeIxAccsBuilder, SetLstFeeIxAccs, SetLstFeeIxArgs, SET_LST_FEE_IX_IS_SIGNER,
    },
    keys::SLAB_ID,
    typedefs::{MintNotFoundErr, SlabEntryPacked},
};
use jiminy_cpi::{
    account::AccountHandle,
    program_error::{ProgramError, INVALID_ACCOUNT_DATA, NOT_ENOUGH_ACCOUNT_KEYS},
};
use sanctum_system_jiminy::sanctum_system_core::instructions::transfer::NewTransferIxAccsBuilder;

use crate::{
    admin_ix_verify_pks_err, admin_ix_verify_signers_err, pay_for_rent_exempt_shortfall,
    verify_pks, verify_signers, Accounts, Cpi, SYS_PROG_ID,
};

pub type SetLstFeeIxAccHandles<'a> = SetLstFeeIxAccs<AccountHandle<'a>>;

pub fn set_lst_fee_accs_checked<'acc>(
    accounts: &Accounts<'acc>,
) -> Result<SetLstFeeIxAccHandles<'acc>, ProgramError> {
    let Some(accs) = accounts.as_slice().first_chunk() else {
        return Err(NOT_ENOUGH_ACCOUNT_KEYS.into());
    };
    let accs = SetLstFeeIxAccHandles::new(*accs);

    let slab = Slab::of_acc_data(accounts.get(*accs.slab()).data()).ok_or(INVALID_ACCOUNT_DATA)?;

    let expected_keys = NewSetLstFeeIxAccsBuilder::start()
        .with_slab(&SLAB_ID)
        .with_system_program(&SYS_PROG_ID)
        .with_admin(slab.admin())
        .with_mint(accounts.get(*accs.mint()).key())
        .with_payer(accounts.get(*accs.payer()).key())
        .build();

    verify_pks(accounts, &accs.0, &expected_keys.0)
        .map_err(|(_actual, expected)| admin_ix_verify_pks_err(expected, slab))?;

    verify_signers(accounts, &accs.0, &SET_LST_FEE_IX_IS_SIGNER.0)
        .map_err(|expected_signer| admin_ix_verify_signers_err(accounts, *expected_signer, slab))?;

    Ok(accs)
}

pub fn process_set_lst_fee<'acc>(
    accounts: &mut Accounts<'acc>,
    accs: SetLstFeeIxAccHandles<'acc>,
    SetLstFeeIxArgs {
        inp_fee_nanos,
        out_fee_nanos,
    }: SetLstFeeIxArgs,
) -> Result<(), ProgramError> {
    let mut cpi = Cpi::new();

    let mint = *accounts.get(*accs.mint()).key();

    let slab_acc = accounts.get_mut(*accs.slab());
    let mut slab = SlabMut::of_acc_data(slab_acc.data_mut()).ok_or(INVALID_ACCOUNT_DATA)?;
    let (_, mut entries) = slab.as_mut();

    match entries.find_by_mint_mut(&mint) {
        Ok(entry) => {
            entry.set_inp_fee_nanos(inp_fee_nanos);
            entry.set_out_fee_nanos(out_fee_nanos);
        }
        Err(MintNotFoundErr { expected_i, mint }) => {
            // grow acc
            let old_acc_len = slab_acc.data_len();
            slab_acc.grow_by(size_of::<SlabEntryPacked>(), false)?;
            let byte_offset = Slab::entry_byte_offset(expected_i);
            slab_acc.data_mut().copy_within(
                byte_offset..old_acc_len,
                byte_offset + size_of::<SlabEntryPacked>(),
            );
            let new_acc_len = slab_acc.data_len();

            pay_for_rent_exempt_shortfall(
                accounts,
                &mut cpi,
                NewTransferIxAccsBuilder::start()
                    .with_from(*accs.payer())
                    .with_to(*accs.slab())
                    .build(),
                new_acc_len,
            )?;

            let mut slab = SlabMut::of_acc_data(accounts.get_mut(*accs.slab()).data_mut())
                .ok_or(INVALID_ACCOUNT_DATA)?;
            let (_, entries) = slab.as_mut();
            let entry = &mut entries.0.get_mut(expected_i).ok_or(INVALID_ACCOUNT_DATA)?;
            *entry.mint_mut() = mint;
            entry.set_inp_fee_nanos(inp_fee_nanos);
            entry.set_out_fee_nanos(out_fee_nanos);
        }
    };
    Ok(())
}
