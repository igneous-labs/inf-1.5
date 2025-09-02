use inf1_ctl_jiminy::{
    accounts::pool_state::PoolState,
    err::Inf1CtlErr,
    program_err::Inf1CtlCustomProgErr,
    typedefs::{lst_state::LstState, u8bool::U8Bool},
};
use jiminy_cpi::{
    account::AccountHandle,
    program_error::{ProgramError, INVALID_ARGUMENT},
};

use crate::Accounts;

#[inline]
pub fn verify_pks<'a, 'acc, const LEN: usize>(
    accounts: &Accounts<'acc>,
    handles: &'a [AccountHandle<'acc>; LEN],
    expected: &'a [&[u8; 32]; LEN], // we can use &[u8; 32] instead of [u8; 32] here bec we dont have any dynamic PDAs to verify
) -> Result<(), (&'a AccountHandle<'acc>, &'a [u8; 32])> {
    verify_pks_slice(accounts, handles, expected)
}

/// [`verify_pks`] delegates to this to minimize monomorphization  
#[inline]
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

#[inline]
pub fn wrong_acc_logmapper<'a, 'acc>(
    accounts: &'a Accounts<'acc>,
) -> impl FnOnce((&AccountHandle<'acc>, &[u8; 32])) -> ProgramError + 'a {
    |(actual, expected)| {
        // dont use format macro to save CUs and binsize
        jiminy_log::sol_log("Wrong account. Expected:");
        jiminy_log::sol_log_pubkey(expected);
        jiminy_log::sol_log("Got:");
        jiminy_log::sol_log_pubkey(accounts.get(*actual).key());
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
pub fn verify_sol_val_calc_prog<'accs>(
    accounts: &Accounts<'accs>,
    lst_state: &LstState,
    prog: AccountHandle<'accs>,
) -> Result<(), ProgramError> {
    let expected = &lst_state.sol_value_calculator;
    if accounts.get(prog).key() != expected {
        Err(wrong_acc_logmapper(accounts)((&prog, expected)))
    } else {
        Ok(())
    }
}
