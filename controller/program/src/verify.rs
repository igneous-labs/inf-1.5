use inf1_ctl_jiminy::{
    accounts::pool_state::PoolState,
    err::Inf1CtlErr,
    program_err::Inf1CtlCustomProgErr,
    typedefs::{lst_state::LstState, u8bool::U8Bool},
};
use jiminy_cpi::{
    account::{Abr, Account, AccountHandle},
    program_error::{BuiltInProgramError, ProgramError, INVALID_ARGUMENT},
};

#[inline]
pub fn verify_pks<'acc, const LEN: usize>(
    abr: &Abr,
    handles: &[AccountHandle<'acc>; LEN],
    expected: &[&[u8; 32]; LEN],
) -> Result<(), ProgramError> {
    verify_pks_pure(abr, handles, expected).map_err(wrong_acc_logmapper(abr))
}

#[inline]
fn verify_pks_pure<'a, 'acc, const LEN: usize>(
    abr: &Abr,
    handles: &'a [AccountHandle<'acc>; LEN],
    expected: &'a [&[u8; 32]; LEN],
) -> Result<(), (&'a AccountHandle<'acc>, &'a [u8; 32])> {
    verify_pks_slice(abr, handles, expected)
}

/// [`verify_pks`] delegates to this to minimize monomorphization,
/// while its const generic LEN ensures both slices are of the same len
#[inline]
fn verify_pks_slice<'a, 'acc>(
    abr: &Abr,
    handles: &'a [AccountHandle<'acc>],
    expected: &'a [&[u8; 32]],
) -> Result<(), (&'a AccountHandle<'acc>, &'a [u8; 32])> {
    handles.iter().zip(expected).try_for_each(|(h, e)| {
        if abr.get(*h).key() == *e {
            Ok(())
        } else {
            Err((h, *e))
        }
    })
}

#[inline]
fn wrong_acc_logmapper<'a, 'acc>(
    abr: &'a Abr,
) -> impl FnOnce((&AccountHandle<'acc>, &[u8; 32])) -> ProgramError + 'a {
    |(actual, expected)| {
        // dont use format macro to save CUs and binsize
        jiminy_log::sol_log("Wrong account. Expected:");
        jiminy_log::sol_log_pubkey(expected);
        jiminy_log::sol_log("Got:");
        jiminy_log::sol_log_pubkey(abr.get(*actual).key());
        // current onchain prog just returns this err, so follow behaviour
        INVALID_ARGUMENT.into()
    }
}

#[inline]
pub fn verify_not_rebalancing_and_not_disabled(pool: &PoolState) -> Result<(), ProgramError> {
    if U8Bool(&pool.is_rebalancing).is_true() {
        return Err(Inf1CtlCustomProgErr(Inf1CtlErr::PoolRebalancing).into());
    }
    if U8Bool(&pool.is_disabled).is_true() {
        return Err(Inf1CtlCustomProgErr(Inf1CtlErr::PoolDisabled).into());
    }
    Ok(())
}

#[inline]
pub fn verify_signers<'a, 'acc, const LEN: usize>(
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

pub fn log_and_return_acc_privilege_err(abr: &Abr, expected_signer: AccountHandle) -> ProgramError {
    jiminy_log::sol_log("Signer privilege escalated for:");
    jiminy_log::sol_log_pubkey(abr.get(expected_signer).key());
    BuiltInProgramError::MissingRequiredSignature.into()
}

#[inline]
pub fn verify_is_program(
    should_be_program: &Account,
    faulty_err: Inf1CtlErr,
) -> Result<(), ProgramError> {
    match should_be_program.is_executable() {
        true => Ok(()),
        false => Err(Inf1CtlCustomProgErr(faulty_err).into()),
    }
}

#[inline]
pub fn verify_sol_value_calculator_is_program(calc_program: &Account) -> Result<(), ProgramError> {
    verify_is_program(calc_program, Inf1CtlErr::FaultySolValueCalculator)
}

pub fn verify_not_input_disabled(lst_state: &LstState) -> Result<(), ProgramError> {
    if U8Bool(&lst_state.is_input_disabled).is_true() {
        return Err(Inf1CtlCustomProgErr(Inf1CtlErr::LstInputDisabled).into());
    }

    Ok(())
}
