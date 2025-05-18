use core::{error::Error, fmt::Display, ops::RangeInclusive};

use inf1_svc_core::traits::SolValCalc;
use inf1_svc_lido_core::calc::{LidoCalc, LidoCalcErr};
use inf1_svc_marinade_core::calc::{MarinadeCalc, MarinadeCalcErr};
use inf1_svc_spl_core::calc::{SplCalc, SplCalcErr};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalcAg {
    Lido(LidoCalc),
    Marinade(MarinadeCalc),
    Spl(SplCalc),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CalcAgErr {
    Lido(LidoCalcErr),
    Marinade(MarinadeCalcErr),
    Spl(SplCalcErr),
}

impl Display for CalcAgErr {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Lido(e) => e.fmt(f),
            Self::Marinade(e) => e.fmt(f),
            Self::Spl(e) => e.fmt(f),
        }
    }
}

impl Error for CalcAgErr {}

impl CalcAg {
    #[inline]
    pub const fn svc_lst_to_sol(&self, lst_amount: u64) -> Result<RangeInclusive<u64>, CalcAgErr> {
        Ok(match self {
            Self::Lido(c) => match c.svc_lst_to_sol(lst_amount) {
                Err(e) => return Err(CalcAgErr::Lido(e)),
                Ok(r) => r,
            },
            Self::Marinade(c) => match c.svc_lst_to_sol(lst_amount) {
                Err(e) => return Err(CalcAgErr::Marinade(e)),
                Ok(r) => r,
            },
            Self::Spl(c) => match c.svc_lst_to_sol(lst_amount) {
                Err(e) => return Err(CalcAgErr::Spl(e)),
                Ok(r) => r,
            },
        })
    }

    #[inline]
    pub const fn svc_sol_to_lst(
        &self,
        lamports_amount: u64,
    ) -> Result<RangeInclusive<u64>, CalcAgErr> {
        Ok(match self {
            Self::Lido(c) => match c.svc_sol_to_lst(lamports_amount) {
                Err(e) => return Err(CalcAgErr::Lido(e)),
                Ok(r) => r,
            },
            Self::Marinade(c) => match c.svc_sol_to_lst(lamports_amount) {
                Err(e) => return Err(CalcAgErr::Marinade(e)),
                Ok(r) => r,
            },
            Self::Spl(c) => match c.svc_sol_to_lst(lamports_amount) {
                Err(e) => return Err(CalcAgErr::Spl(e)),
                Ok(r) => r,
            },
        })
    }
}

impl SolValCalc for CalcAg {
    type Error = CalcAgErr;

    #[inline]
    fn lst_to_sol(&self, lst_amount: u64) -> Result<RangeInclusive<u64>, Self::Error> {
        self.svc_lst_to_sol(lst_amount)
    }

    #[inline]
    fn sol_to_lst(&self, lamports_amount: u64) -> Result<RangeInclusive<u64>, Self::Error> {
        self.svc_sol_to_lst(lamports_amount)
    }
}
