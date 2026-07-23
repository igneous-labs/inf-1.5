use jiminy_cpi::{
    account::{Abr, AccountHandle},
    program_error::{
        BuiltInProgramError, ProgramError, INVALID_ARGUMENT, MISSING_REQUIRED_SIGNATURE,
    },
};
use jiminy_sysvar_rent::{sysvar::SimpleSysvar, Rent};
use sanctum_system_jiminy::instructions::transfer::{transfer_invoke_fwd, TransferIxAccounts};

/// Max CPI accounts needed across all instructions
const MAX_CPI_ACCS: usize = 2;

pub type Cpi = jiminy_cpi::Cpi<MAX_CPI_ACCS>;

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
    verify_pks_slice(abr, handles, expected).map_err(wrong_acc_logmap)
}

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
fn verify_pk_eq<'a>(actual: &'a [u8; 32], expected: &'a [u8; 32]) -> Result<(), [&'a [u8; 32]; 2]> {
    if actual == expected {
        Ok(())
    } else {
        Err([actual, expected])
    }
}

#[inline]
fn wrong_acc_logmap([actual, expected]: [&[u8; 32]; 2]) -> ProgramError {
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
    verify_signers_slice(abr, handles, expected_is_signer)
        .map_err(|expected_signer| log_and_return_acc_privilege_err(abr, *expected_signer))
}

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

pub fn pay_for_rent_exempt_shortfall<'acc>(
    abr: &mut Abr,
    cpi: &mut Cpi,
    handles: TransferIxAccounts<'acc>,
    data_len: usize,
) -> Result<(), ProgramError> {
    let lamports_shortfall = Rent::get()?
        .min_balance(data_len)
        .saturating_sub(abr.get(*handles.to()).lamports());

    if lamports_shortfall > 0 {
        transfer_invoke_fwd(abr, cpi, handles, lamports_shortfall)?;
    }

    Ok(())
}
