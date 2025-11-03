use inf1_ctl_core::{accounts::pool_state::PoolState, err::Inf1CtlErr};
use jiminy_cpi::account::Account;

use crate::program_err::Inf1CtlCustomProgErr;

#[inline]
pub fn pool_state_checked(acc: &Account) -> Result<&PoolState, Inf1CtlCustomProgErr> {
    // safety: account data is 8-byte aligned
    unsafe { PoolState::of_acc_data(acc.data()) }
        .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidPoolStateData))
}

#[inline]
pub fn pool_state_checked_mut(acc: &mut Account) -> Result<&mut PoolState, Inf1CtlCustomProgErr> {
    // safety: account data is 8-byte aligned
    unsafe { PoolState::of_acc_data_mut(acc.data_mut()) }
        .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidPoolStateData))
}
