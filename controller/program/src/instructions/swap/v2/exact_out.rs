use inf1_ctl_jiminy::{account_utils::pool_state_v2_checked_mut, instructions::swap::IxArgs};
use jiminy_cpi::{account::Abr, program_error::ProgramError};
use jiminy_sysvar_clock::Clock;

use crate::{
    instructions::swap::v2::{initial_pair_sync, SwapV2IxAccounts, SwapV2Ty},
    yield_release::release_yield,
    Cpi,
};

#[inline]
pub fn process_swap_exact_out_v2(
    abr: &mut Abr,
    cpi: &mut Cpi,
    accs: &SwapV2IxAccounts,
    args: &IxArgs,
    ty: SwapV2Ty,
    clock: &Clock,
) -> Result<(), ProgramError> {
    let pool = pool_state_v2_checked_mut(abr.get_mut(*accs.ix_prefix.pool_state()))?;
    release_yield(pool, clock)?;

    initial_pair_sync(abr, cpi, accs, args, ty)?;

    Ok(())
}
