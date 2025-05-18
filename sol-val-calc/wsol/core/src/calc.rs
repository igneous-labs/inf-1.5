use core::{convert::Infallible, ops::RangeInclusive};

use inf1_svc_core::traits::SolValCalc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WsolCalc;

impl SolValCalc for WsolCalc {
    type Error = Infallible;

    #[inline]
    fn lst_to_sol(&self, lst_amount: u64) -> Result<RangeInclusive<u64>, Self::Error> {
        Ok(lst_amount..=lst_amount)
    }

    #[inline]
    fn sol_to_lst(&self, lamports_amount: u64) -> Result<RangeInclusive<u64>, Self::Error> {
        Ok(lamports_amount..=lamports_amount)
    }
}
