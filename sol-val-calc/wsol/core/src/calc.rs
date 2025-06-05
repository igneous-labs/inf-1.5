use core::{convert::Infallible, ops::RangeInclusive};

use inf1_svc_core::traits::SolValCalc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WsolCalc;

impl WsolCalc {
    #[inline]
    pub const fn svc_lst_to_sol(&self, lst_amount: u64) -> RangeInclusive<u64> {
        lst_amount..=lst_amount
    }

    #[inline]
    pub const fn svc_sol_to_lst(&self, lamports_amount: u64) -> RangeInclusive<u64> {
        lamports_amount..=lamports_amount
    }
}

impl SolValCalc for WsolCalc {
    type Error = Infallible;

    #[inline]
    fn lst_to_sol(&self, lst_amount: u64) -> Result<RangeInclusive<u64>, Self::Error> {
        Ok(self.svc_lst_to_sol(lst_amount))
    }

    #[inline]
    fn sol_to_lst(&self, lamports_amount: u64) -> Result<RangeInclusive<u64>, Self::Error> {
        Ok(self.svc_sol_to_lst(lamports_amount))
    }
}
