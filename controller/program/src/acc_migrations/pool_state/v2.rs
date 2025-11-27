#![allow(unused)] // DELETEME: once we start using migrate_idmpt()

use core::cmp::max;

use inf1_ctl_jiminy::{
    account_utils::{pool_state_checked, pool_state_v2_checked},
    accounts::pool_state::{PoolState, PoolStateV2},
    err::Inf1CtlErr,
    keys::POOL_STATE_ID,
    program_err::Inf1CtlCustomProgErr,
    typedefs::{fee_nanos::NANOS_DENOM, rps::Rps},
};
use jiminy_cpi::{account::Account, program_error::ProgramError};
use jiminy_sysvar_clock::Clock;

const BPS_TO_NANOS_MULTIPLE: u32 = NANOS_DENOM / 10_000;

/// Also verifies identity of `pool_state_acc_unchecked`
///
/// # Prerequisites
/// - pool state account must have enough SOL for rent exemption of new extended length
#[inline]
pub fn migrate_idmpt(
    pool_state_acc_unchecked: &mut Account,
    clock: &Clock,
) -> Result<(), ProgramError> {
    if *pool_state_acc_unchecked.key() != POOL_STATE_ID {
        return Err(Inf1CtlCustomProgErr(Inf1CtlErr::IncorrectPoolState).into());
    }

    // need to use bool here bec pool_state_v2_checked() borrows account, blocking mutation
    if pool_state_v2_checked(pool_state_acc_unchecked).is_err() {
        // this also ensures pool_state is v1 at this point
        let init_protocol_fee_nanos = pool_state_checked(pool_state_acc_unchecked).map(
            |PoolState {
                 trading_protocol_fee_bps,
                 lp_protocol_fee_bps,
                 ..
             }| {
                // unchecked-arith safety: valid bps < 10_000, no overflow from mul
                u32::from(*max(trading_protocol_fee_bps, lp_protocol_fee_bps))
                    * BPS_TO_NANOS_MULTIPLE
            },
        )?;

        pool_state_acc_unchecked.realloc(core::mem::size_of::<PoolStateV2>(), false)?;
        let PoolStateV2 {
            protocol_fee_nanos,
            version,
            admin,
            rps_authority,
            rps,
            withheld_lamports,
            protocol_fee_lamports,
            last_release_slot,
            ..
        } = unsafe { PoolStateV2::of_acc_data_mut(pool_state_acc_unchecked.data_mut()) }
            .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidPoolStateData))?;

        *version = 2;
        *protocol_fee_nanos = init_protocol_fee_nanos;
        *rps_authority = *admin;
        *withheld_lamports = 0;
        *protocol_fee_lamports = 0;
        *rps = *Rps::DEFAULT.as_raw();
        *last_release_slot = clock.slot;
    }

    Ok(())
}
