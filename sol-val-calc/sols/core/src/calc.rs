use core::{convert::Infallible, ops::RangeInclusive};

use inf1_svc_core::traits::SolValCalc;

/// Returns nominal 1:1
///
/// ## Notes
/// - if the SOLS pool is currently out of SOL liquidity, the SOLS token might
///   only be redeemable for holding LSTs below par via ClaimHolding at that point in time.
///   Possible sources of loss of SOL value resulting in below par value:
///   - `sol_out_loss_tol`
///   - holding LSTs experience SOL value loss due to slashing or other means
///   - SOLS program protocol fees
///
///   Each of these represents a possibly permanent loss of SOL value for the INF pool if the
///   SOLS pool manager/rebalancer is unwilling to keep the pool whole by rebalancing impaired holdings
///   back into SOL. The manager must therefore whitelist SOLS pools according to strict discretion
///   and quickly remove SOLS tokens from such impaired pools.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SolsCalc;

/// `SolValCalc`
impl SolsCalc {
    #[inline]
    pub const fn svc_lst_to_sol(&self, lst_amount: u64) -> Result<RangeInclusive<u64>, Infallible> {
        Ok(lst_amount..=lst_amount)
    }

    #[inline]
    pub const fn svc_sol_to_lst(
        &self,
        lamports_amount: u64,
    ) -> Result<RangeInclusive<u64>, Infallible> {
        Ok(lamports_amount..=lamports_amount)
    }
}

impl SolValCalc for SolsCalc {
    type Error = Infallible;

    #[inline]
    fn lst_to_sol(&self, lst_amount: u64) -> Result<RangeInclusive<u64>, Self::Error> {
        self.svc_lst_to_sol(lst_amount)
    }

    #[inline]
    fn sol_to_lst(&self, lamports_amount: u64) -> Result<RangeInclusive<u64>, Self::Error> {
        self.svc_sol_to_lst(lamports_amount)
    }
}
