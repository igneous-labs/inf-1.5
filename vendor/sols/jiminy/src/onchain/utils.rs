use sanctum_sols_core::{
    err::SanctumSolsErr,
    utils::{vercomp_mint_claim_amt, LamportSurplusArgs},
};

use crate::onchain::err::SanctumSolsProgErr;

/// [`vercomp_mint_claim_amt`] but with err converted for easier interop
/// with [`jiminy_program_error::ProgramError`]
#[inline]
pub const fn prog_vercomp_mint_claim_amt(
    amt: Option<u64>,
    args: &LamportSurplusArgs,
) -> Result<u64, SanctumSolsProgErr> {
    match vercomp_mint_claim_amt(amt, args) {
        Err(e) => Err(SanctumSolsProgErr(SanctumSolsErr::NotEnoughLiquidity(e))),
        Ok(x) => Ok(x),
    }
}
