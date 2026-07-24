use jiminy_cpi::{
    account::{Abr, AccountHandle},
    program_error::{
        BuiltInProgramError, ProgramError, ILLEGAL_OWNER, INVALID_ARGUMENT,
        INVALID_INSTRUCTION_DATA, MISSING_REQUIRED_SIGNATURE,
    },
};

use jiminy_sysvar_rent::{sysvar::SimpleSysvar, Rent};
use sanctum_system_jiminy::instructions::transfer::{transfer_invoke_fwd, TransferIxAccounts};

use crate::Cpi;

/// accounts split first chunk
#[inline]
pub const fn asfc<'a, 'acc, const N: usize>(
    accs: &'a [AccountHandle<'acc>],
) -> Result<(&'a [AccountHandle<'acc>; N], &'a [AccountHandle<'acc>]), ProgramError> {
    match accs.split_first_chunk() {
        Some(x) => Ok(x),
        None => Err(ProgramError::from_builtin(
            BuiltInProgramError::NotEnoughAccountKeys,
        )),
    }
}

/// ix data cast.
/// Casts instruction data into const array slice
#[inline]
pub fn ixdc<const N: usize>(ix_data: &[u8]) -> Result<&[u8; N], ProgramError> {
    ix_data
        .try_into()
        .map_err(|_| INVALID_INSTRUCTION_DATA.into())
}

#[inline]
pub fn verify_pks<'acc, const LEN: usize>(
    abr: &Abr,
    handles: &[AccountHandle<'acc>; LEN],
    expected: &[&[u8; 32]; LEN],
) -> Result<(), ProgramError> {
    verify_pks_slice(abr, handles, expected).map_err(|e| {
        log_wrong_acc(e);
        INVALID_ARGUMENT.into()
    })
}

#[inline]
pub fn verify_owners<'acc, const LEN: usize>(
    abr: &Abr,
    handles: &[AccountHandle<'acc>; LEN],
    expected: &[&[u8; 32]; LEN],
) -> Result<(), ProgramError> {
    verify_owners_slice(abr, handles, expected).map_err(|e| {
        log_wrong_acc(e);
        ILLEGAL_OWNER.into()
    })
}

#[inline]
fn verify_pks_slice<'a>(
    abr: &'a Abr,
    handles: &[AccountHandle],
    expected: &[&'a [u8; 32]],
) -> Result<(), [&'a [u8; 32]; 2]> {
    handles
        .iter()
        .map(|h| abr.get(*h).key())
        .zip(expected)
        .try_for_each(|(a, e)| verify_pk_eq(a, e))
}

#[inline]
fn verify_owners_slice<'a>(
    abr: &'a Abr,
    handles: &[AccountHandle],
    expected: &[&'a [u8; 32]],
) -> Result<(), [&'a [u8; 32]; 2]> {
    handles
        .iter()
        .map(|h| abr.get(*h).owner())
        .zip(expected)
        .try_for_each(|(a, e)| verify_pk_eq(a, e))
}

#[inline]
fn verify_pk_eq<'a>(actual: &'a [u8; 32], expected: &'a [u8; 32]) -> Result<(), [&'a [u8; 32]; 2]> {
    if actual == expected {
        Ok(())
    } else {
        Err([actual, expected])
    }
}

fn log_wrong_acc([actual, expected]: [&[u8; 32]; 2]) {
    jiminy_log::sol_log("Wrong account. Expected:");
    jiminy_log::sol_log_pubkey(expected);
    jiminy_log::sol_log("Got:");
    jiminy_log::sol_log_pubkey(actual);
}

#[inline]
pub fn verify_signers<'a, 'acc, const LEN: usize>(
    abr: &Abr,
    handles: &'a [AccountHandle<'acc>; LEN],
    expected_is_signer: &'a [bool; LEN],
) -> Result<(), ProgramError> {
    verify_signers_pure(abr, handles, expected_is_signer)
        .map_err(|expected_signer| log_and_return_acc_privilege_err(abr, *expected_signer))
}

/// Returns first offending AccountHandle of account that was
/// expected to be a signer but was not
#[inline]
fn verify_signers_pure<'a, 'acc, const LEN: usize>(
    abr: &Abr,
    handles: &'a [AccountHandle<'acc>; LEN],
    expected_is_signer: &'a [bool; LEN],
) -> Result<(), &'a AccountHandle<'acc>> {
    verify_signers_slice(abr, handles, expected_is_signer)
}

/// [`verify_signers`] delegates to this to minimize monomorphization
#[inline]
fn verify_signers_slice<'a, 'acc>(
    abr: &Abr,
    handles: &'a [AccountHandle<'acc>],
    expected_is_signer: &'a [bool],
) -> Result<(), &'a AccountHandle<'acc>> {
    handles
        .iter()
        .zip(expected_is_signer)
        .try_for_each(|(h, should_be_signer)| {
            if *should_be_signer && !abr.get(*h).is_signer() {
                Err(h)
            } else {
                Ok(())
            }
        })
}

#[inline]
fn log_and_return_acc_privilege_err(abr: &Abr, expected_signer: AccountHandle) -> ProgramError {
    jiminy_log::sol_log("Signer privilege escalated for:");
    jiminy_log::sol_log_pubkey(abr.get(expected_signer).key());
    MISSING_REQUIRED_SIGNATURE.into()
}

#[inline]
pub fn pay_for_rent_exempt_shortfall<'acc>(
    abr: &mut Abr,
    cpi: &mut Cpi,
    handles: TransferIxAccounts<'acc>,
) -> Result<(), ProgramError> {
    let data_len = abr.get(*handles.to()).data().len();
    let lamports_shortfall = Rent::get()?
        .min_balance(data_len)
        .saturating_sub(abr.get(*handles.to()).lamports());

    if lamports_shortfall > 0 {
        transfer_invoke_fwd(abr, cpi, handles, lamports_shortfall)?;
    }

    Ok(())
}
