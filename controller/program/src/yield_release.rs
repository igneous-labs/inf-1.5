use inf1_ctl_jiminy::{
    accounts::pool_state::PoolStateV2,
    err::Inf1CtlErr,
    program_err::Inf1CtlCustomProgErr,
    yields::release::{ReleaseYield, YRelLamports},
};
use jiminy_cpi::program_error::ProgramError;
use jiminy_sysvar_clock::Clock;

/// TODO: use return value to create yield release event for self-cpi logging
pub fn release_yield(ps: &mut PoolStateV2, clock: &Clock) -> Result<YRelLamports, ProgramError> {
    let yrel = ReleaseYield::new(ps, clock.slot)
        .map_err(Inf1CtlCustomProgErr)?
        .calc();
    ps.apply_yrel(yrel, clock.slot)
        .ok_or(Inf1CtlCustomProgErr(Inf1CtlErr::MathError))?;
    Ok(yrel)
}
