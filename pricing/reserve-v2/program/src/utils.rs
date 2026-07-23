use jiminy_cpi::{
    account::{Abr, AccountHandle},
    program_error::{BuiltInProgramError, ProgramError, ILLEGAL_OWNER, INVALID_ARGUMENT},
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

#[inline]
fn log_wrong_acc([actual, expected]: [&[u8; 32]; 2]) {
    jiminy_log::sol_log("Wrong account. Expected:");
    jiminy_log::sol_log_pubkey(expected);
    jiminy_log::sol_log("Got:");
    jiminy_log::sol_log_pubkey(actual);
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
