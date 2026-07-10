use jiminy_account::{Abr, AccountHandle};
use jiminy_cpi::program_error::{ProgramError, INVALID_ARGUMENT, MISSING_REQUIRED_SIGNATURE};

#[inline]
pub fn verify_pks<'acc, const LEN: usize>(
    abr: &Abr,
    handles: &[AccountHandle<'acc>; LEN],
    expected: &[&[u8; 32]; LEN],
) -> Result<(), ProgramError> {
    verify_pks_pure(abr, handles, expected).map_err(wrong_acc_logmap)
}

#[inline]
fn verify_pks_pure<'a, const LEN: usize>(
    abr: &'a Abr,
    handles: &[AccountHandle; LEN],
    expected: &[&'a [u8; 32]; LEN],
) -> Result<(), [&'a [u8; 32]; 2]> {
    verify_pks_slice(abr, handles, expected)
}

/// [`verify_pks`] delegates to this to minimize monomorphization,
/// while its const generic LEN ensures both slices are of the same len
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
pub fn verify_pks_raw<'a, const LEN: usize>(
    actual: &[&'a [u8; 32]; LEN],
    expected: &[&'a [u8; 32]; LEN],
) -> Result<(), ProgramError> {
    verify_pks_raw_pure(actual, expected).map_err(wrong_acc_logmap)
}

#[inline]
fn verify_pks_raw_pure<'a, const LEN: usize>(
    actual: &[&'a [u8; 32]; LEN],
    expected: &[&'a [u8; 32]; LEN],
) -> Result<(), [&'a [u8; 32]; 2]> {
    verify_pks_raw_slice(actual, expected)
}

/// [`verify_pks_raw`] delegates to this to minimize monomorphization,
/// while its const generic LEN ensures both slices are of the same len
#[inline]
fn verify_pks_raw_slice<'a>(
    actual: &[&'a [u8; 32]],
    expected: &[&'a [u8; 32]],
) -> Result<(), [&'a [u8; 32]; 2]> {
    actual
        .iter()
        .zip(expected)
        .try_for_each(|(a, e)| verify_pk_eq(a, e))
}

/// On err returns Err([actual, expected])
#[inline]
fn verify_pk_eq<'a>(actual: &'a [u8; 32], expected: &'a [u8; 32]) -> Result<(), [&'a [u8; 32]; 2]> {
    if actual == expected {
        Ok(())
    } else {
        Err([actual, expected])
    }
}

#[inline]
fn wrong_acc_logmap([actual, expected]: [&[u8; 32]; 2]) -> ProgramError {
    // dont use format macro to save CUs and binsize
    jiminy_log::sol_log("Wrong account. Expected:");
    jiminy_log::sol_log_pubkey(expected);
    jiminy_log::sol_log("Got:");
    jiminy_log::sol_log_pubkey(actual);
    INVALID_ARGUMENT.into()
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

#[inline]
fn verify_signers_pure<'a, 'acc, const LEN: usize>(
    abr: &Abr,
    handles: &'a [AccountHandle<'acc>; LEN],
    expected_is_signer: &'a [bool; LEN],
) -> Result<(), &'a AccountHandle<'acc>> {
    verify_signers_slice(abr, handles, expected_is_signer)
}

/// [`verify_signers`] delegates to this to minimize monomorphization
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

fn log_and_return_acc_privilege_err(abr: &Abr, expected_signer: AccountHandle) -> ProgramError {
    jiminy_log::sol_log("Signer privilege escalated for:");
    jiminy_log::sol_log_pubkey(abr.get(expected_signer).key());
    MISSING_REQUIRED_SIGNATURE.into()
}
