use inf1_pp_flatslab_core::{accounts::Slab, errs::FlatSlabProgramErr, keys::SLAB_ID};
use jiminy_cpi::program_error::{ProgramError, INVALID_ARGUMENT};
use jiminy_entrypoint::account::AccountHandle;
use jiminy_system_prog_interface::{transfer_ix, TransferIxAccounts, TransferIxData};
use jiminy_sysvar_rent::{sysvar::SimpleSysvar, Rent};

use crate::{Accounts, CustomProgErr};

/// SystemInstruction::transfer
const MAX_CPI_ACCS: usize = 2;

pub type Cpi = jiminy_cpi::Cpi<MAX_CPI_ACCS>;

pub const SYS_PROG_ID: [u8; 32] = [0u8; 32];

#[inline]
pub fn verify_pks<'a, 'acc, const LEN: usize>(
    accounts: &Accounts<'acc>,
    handles: &'a [AccountHandle<'acc>; LEN],
    expected: &'a [&[u8; 32]; LEN], // we can use &[u8; 32] instead of [u8; 32] here bec we dont have any dynamic PDAs to verify
) -> Result<(), (&'a AccountHandle<'acc>, &'a [u8; 32])> {
    verify_pks_slice(accounts, handles, expected)
}

/// [`verify_pks`] delegates to this to minimize monomorphization  
fn verify_pks_slice<'a, 'acc>(
    accounts: &Accounts<'acc>,
    handles: &'a [AccountHandle<'acc>],
    expected: &'a [&[u8; 32]],
) -> Result<(), (&'a AccountHandle<'acc>, &'a [u8; 32])> {
    handles.iter().zip(expected).try_for_each(|(h, e)| {
        if accounts.get(*h).key() == *e {
            Ok(())
        } else {
            Err((h, *e))
        }
    })
}

pub fn admin_ix_verify_pks_err(expected: &[u8; 32], slab: Slab) -> ProgramError {
    if *expected == SLAB_ID {
        ProgramError::from(CustomProgErr(FlatSlabProgramErr::WrongSlabAcc))
    } else if expected == slab.admin() {
        CustomProgErr(FlatSlabProgramErr::MissingAdminSignature).into()
    } else {
        INVALID_ARGUMENT.into()
    }
}

#[inline]
pub fn verify_signers<'a, 'acc, const LEN: usize>(
    accounts: &Accounts<'acc>,
    handles: &'a [AccountHandle<'acc>; LEN],
    expected_is_signer: &'a [bool],
) -> Result<(), &'a AccountHandle<'acc>> {
    verify_signers_slice(accounts, handles, expected_is_signer)
}

/// [`verify_signers`] delegates to this to minimize monomorphization
fn verify_signers_slice<'a, 'acc>(
    accounts: &Accounts<'acc>,
    handles: &'a [AccountHandle<'acc>],
    expected_is_signer: &'a [bool],
) -> Result<(), &'a AccountHandle<'acc>> {
    handles
        .iter()
        .zip(expected_is_signer)
        .try_for_each(|(h, should_be_signer)| {
            if *should_be_signer && !accounts.get(*h).is_signer() {
                Err(h)
            } else {
                Ok(())
            }
        })
}

pub fn admin_ix_verify_signers_err(
    accounts: &Accounts,
    expected_signer: AccountHandle,
    slab: Slab,
) -> ProgramError {
    if accounts.get(expected_signer).key() == slab.admin() {
        CustomProgErr(FlatSlabProgramErr::MissingAdminSignature).into()
    } else {
        INVALID_ARGUMENT.into()
    }
}

pub fn pay_for_rent_exempt_shortfall<'acc>(
    accounts: &mut Accounts<'acc>,
    cpi: &mut Cpi,
    system_prog: AccountHandle<'acc>,
    handles: TransferIxAccounts<'acc>,
    data_len: usize,
) -> Result<(), ProgramError> {
    let lamports_shortfall = Rent::get()?
        .min_balance(data_len)
        .saturating_sub(accounts.get(*handles.to()).lamports());

    if lamports_shortfall > 0 {
        cpi.invoke_signed(
            accounts,
            transfer_ix(
                system_prog,
                handles,
                &TransferIxData::new(lamports_shortfall),
            ),
            &[],
        )?;
    }

    Ok(())
}
