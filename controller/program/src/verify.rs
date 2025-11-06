use inf1_core::typedefs::fee_bps::fee_bps;
use inf1_ctl_jiminy::{
    accounts::pool_state::PoolState,
    err::Inf1CtlErr,
    keys::{TOKENKEG_ID, TOKEN_2022_ID},
    program_err::Inf1CtlCustomProgErr,
    typedefs::lst_state::LstState,
    typedefs::u8bool::U8Bool,
};
use jiminy_cpi::{
    account::{Abr, Account, AccountHandle},
    program_error::{
        ProgramError, ILLEGAL_OWNER, INVALID_ACCOUNT_DATA, INVALID_ARGUMENT,
        MISSING_REQUIRED_SIGNATURE,
    },
};
use sanctum_spl_token_jiminy::sanctum_spl_token_core::state::mint::{Mint, RawMint};

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
pub fn verify_is_rebalancing(pool: &PoolState) -> Result<(), ProgramError> {
    if U8Bool(&pool.is_rebalancing).is_false() {
        return Err(Inf1CtlCustomProgErr(Inf1CtlErr::PoolNotRebalancing).into());
    }
    Ok(())
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

#[inline]
pub fn verify_tokenkeg_or_22_mint(mint: &Account) -> Result<(), ProgramError> {
    if *mint.owner() != TOKENKEG_ID && *mint.owner() != TOKEN_2022_ID {
        return Err(ILLEGAL_OWNER.into());
    }

    // Verify mint is initialized
    RawMint::of_acc_data(mint.data())
        .and_then(Mint::try_from_raw)
        .ok_or(INVALID_ACCOUNT_DATA)?;

    Ok(())
}

#[inline]
pub fn verify_pricing_program_is_program(pricing_program: &Account) -> Result<(), ProgramError> {
    verify_is_program(pricing_program, Inf1CtlErr::FaultyPricingProgram)
}

#[inline]
pub fn verify_valid_fee_bps(bps: u16) -> Result<(), ProgramError> {
    fee_bps(bps)
        .ok_or_else(|| Inf1CtlCustomProgErr(Inf1CtlErr::FeeTooHigh).into())
        .map(|_| ())
}

/// Perform a linear search to verify that no existing entries
/// on `list` has the same key as `key`. Else returns `err`.
#[inline]
fn verify_list_no_dup_by_key<T, K: PartialEq>(
    list: &[T],
    key: &K,
    key_fn: impl for<'a> Fn(&'a T) -> &'a K,
    err: Inf1CtlErr,
) -> Result<(), ProgramError> {
    match list.iter().find(|existing| key_fn(existing) == key) {
        None => Ok(()),
        Some(_) => Err(Inf1CtlCustomProgErr(err).into()),
    }
}

#[inline]
pub fn verify_disable_pool_auth_list_no_dup(
    list: &[[u8; 32]],
    new_auth: &[u8; 32],
) -> Result<(), ProgramError> {
    verify_list_no_dup_by_key(
        list,
        new_auth,
        |pk| pk,
        Inf1CtlErr::DuplicateDisablePoolAuthority,
    )
}
