use core::{convert::Infallible, ops::RangeInclusive};

use inf1_svc_core::traits::SolValCalc;
use inf1_svc_inf_core::{InfCalc, InfCalcErr};
use inf1_svc_lido_core::calc::{LidoCalc, LidoCalcErr};
use inf1_svc_marinade_core::calc::{MarinadeCalc, MarinadeCalcErr};
use inf1_svc_spl_core::calc::{SplCalc, SplCalcErr};
use inf1_svc_wsol_core::calc::WsolCalc;

use crate::{each_fallible_variant_method, SvcAg};

pub type SvcCalcAg = SvcAg<InfCalc, LidoCalc, MarinadeCalc, SplCalc, SplCalc, SplCalc, WsolCalc>;

pub type SvcCalcAgRef<'a> = SvcAg<
    &'a InfCalc,
    &'a LidoCalc,
    &'a MarinadeCalc,
    &'a SplCalc,
    &'a SplCalc,
    &'a SplCalc,
    &'a WsolCalc,
>;

pub type SvcCalcAgErr =
    SvcAg<InfCalcErr, LidoCalcErr, MarinadeCalcErr, SplCalcErr, SplCalcErr, SplCalcErr, Infallible>;

impl SvcCalcAgRef<'_> {
    #[inline]
    pub const fn svc_lst_to_sol(
        &self,
        lst_amount: u64,
    ) -> Result<RangeInclusive<u64>, SvcCalcAgErr> {
        each_fallible_variant_method!(self, svc_lst_to_sol(lst_amount))
    }

    #[inline]
    pub const fn svc_sol_to_lst(
        &self,
        lamports_amount: u64,
    ) -> Result<RangeInclusive<u64>, SvcCalcAgErr> {
        each_fallible_variant_method!(self, svc_sol_to_lst(lamports_amount))
    }
}

impl SolValCalc for SvcCalcAgRef<'_> {
    type Error = SvcCalcAgErr;

    #[inline]
    fn lst_to_sol(&self, lst_amount: u64) -> Result<RangeInclusive<u64>, Self::Error> {
        self.svc_lst_to_sol(lst_amount)
    }

    #[inline]
    fn sol_to_lst(&self, lamports_amount: u64) -> Result<RangeInclusive<u64>, Self::Error> {
        self.svc_sol_to_lst(lamports_amount)
    }
}

impl SolValCalc for SvcCalcAg {
    type Error = SvcCalcAgErr;

    #[inline]
    fn lst_to_sol(&self, lst_amount: u64) -> Result<RangeInclusive<u64>, Self::Error> {
        self.as_ref_const().svc_lst_to_sol(lst_amount)
    }

    #[inline]
    fn sol_to_lst(&self, lamports_amount: u64) -> Result<RangeInclusive<u64>, Self::Error> {
        self.as_ref_const().svc_sol_to_lst(lamports_amount)
    }
}
