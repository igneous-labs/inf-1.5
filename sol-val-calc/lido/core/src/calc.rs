use core::{error::Error, fmt::Display, ops::RangeInclusive};

use inf1_svc_core::traits::SolValCalc;
use sanctum_token_ratio_compat::floor_ratio_u64_u64_reverse;
use solido_legacy_core::{ExchangeRate, Lido};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LidoCalc {
    pub exchange_rate: ExchangeRate,
    pub current_epoch: u64,
}

/// Constructors
impl LidoCalc {
    #[inline]
    pub const fn new(Lido { exchange_rate, .. }: &Lido, current_epoch: u64) -> Self {
        Self {
            exchange_rate: *exchange_rate,
            current_epoch,
        }
    }
}

/// SolValCalc
impl LidoCalc {
    #[inline]
    pub const fn is_updated(&self) -> bool {
        self.exchange_rate.computed_in_epoch >= self.current_epoch
    }

    #[inline]
    pub const fn svc_lst_to_sol(
        &self,
        lst_amount: u64,
    ) -> Result<RangeInclusive<u64>, LidoCalcErr> {
        if !self.is_updated() {
            return Err(LidoCalcErr::NotUpdated);
        }
        match self
            .exchange_rate
            .sol_balance_over_st_sol_supply()
            .apply(lst_amount)
        {
            Some(val) => Ok(val..=val),
            None => Err(LidoCalcErr::Ratio),
        }
    }

    #[inline]
    pub const fn svc_sol_to_lst(
        &self,
        lamports_amount: u64,
    ) -> Result<RangeInclusive<u64>, LidoCalcErr> {
        if !self.is_updated() {
            return Err(LidoCalcErr::NotUpdated);
        }
        match floor_ratio_u64_u64_reverse(
            self.exchange_rate.sol_balance_over_st_sol_supply(),
            lamports_amount,
        ) {
            Some(r) => Ok(r),
            None => Err(LidoCalcErr::Ratio),
        }
    }
}

impl SolValCalc for LidoCalc {
    type Error = LidoCalcErr;

    #[inline]
    fn lst_to_sol(&self, lst_amount: u64) -> Result<RangeInclusive<u64>, Self::Error> {
        self.svc_lst_to_sol(lst_amount)
    }

    #[inline]
    fn sol_to_lst(&self, lamports_amount: u64) -> Result<RangeInclusive<u64>, Self::Error> {
        self.svc_sol_to_lst(lamports_amount)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LidoCalcErr {
    Ratio,
    NotUpdated,
}

impl Display for LidoCalcErr {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::NotUpdated => "not yet updated this epoch",
            Self::Ratio => "ratio math error",
        })
    }
}

impl Error for LidoCalcErr {}
