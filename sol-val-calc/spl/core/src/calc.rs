use core::{error::Error, fmt::Display, ops::RangeInclusive};

use inf1_svc_core::traits::SolValCalc;
use sanctum_fee_ratio::ratio::{Ceil, Ratio};
use sanctum_spl_stake_pool_core::{Fee, StakePool};
use sanctum_token_ratio_compat::{
    fee_ceil_ratio_u64_u64_reverse_from_rem, floor_ratio_u64_u64_reverse,
};
use sanctum_u64_ratio::Floor;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SplCalc {
    pub last_update_epoch: u64,
    pub total_lamports: u64,
    pub pool_token_supply: u64,
    pub stake_withdrawal_fee: Fee,
    pub current_epoch: u64,
}

/// Constructors
impl SplCalc {
    #[inline]
    pub const fn new(
        StakePool {
            last_update_epoch,
            total_lamports,
            pool_token_supply,
            stake_withdrawal_fee,
            ..
        }: &StakePool,
        current_epoch: u64,
    ) -> Self {
        Self {
            last_update_epoch: *last_update_epoch,
            total_lamports: *total_lamports,
            pool_token_supply: *pool_token_supply,
            stake_withdrawal_fee: *stake_withdrawal_fee,
            current_epoch,
        }
    }
}

type Fcr = sanctum_fee_ratio::Fee<Ceil<Ratio<u64, u64>>>;

/// SolValCalc
///
/// Assumes:
/// - stake pool manager is always valid, so stake withdraw fee will always be charged
/// - stake pool always has active and transient stake, so withdraw_source != StakeWithdrawSource::ValidatorRemoval
impl SplCalc {
    #[inline]
    pub const fn is_updated(&self) -> bool {
        self.last_update_epoch >= self.current_epoch
    }

    #[inline]
    pub const fn stake_withdrawal_fee_ceil(&self) -> Option<Fcr> {
        let Fee {
            denominator: d,
            numerator: n,
        } = self.stake_withdrawal_fee;
        // The SPL stake pool program permits denominator to = 0
        // (treated as 0 fee in that case)
        // But sanctum_fee_ratio does not, so we need to
        // preprocess all 0 denom fees
        let ratio = if d == 0 {
            Ratio { n: 0, d: 1 }
        } else {
            Ratio { n, d }
        };
        Fcr::new(ratio)
    }

    #[inline]
    pub const fn lst_to_lamports_ratio(&self) -> Floor<Ratio<u64, u64>> {
        let Self {
            total_lamports,
            pool_token_supply,
            ..
        } = self;
        Floor(Ratio {
            n: *total_lamports,
            d: *pool_token_supply,
        })
    }

    #[inline]
    pub const fn svc_lst_to_sol(&self, lst_amount: u64) -> Result<RangeInclusive<u64>, SplCalcErr> {
        if !self.is_updated() {
            return Err(SplCalcErr::NotUpdated);
        }
        let fee = match self.stake_withdrawal_fee_ceil() {
            Some(f) => f,
            None => return Err(SplCalcErr::Ratio),
        };
        let aaf = match fee.apply(lst_amount) {
            Some(a) => a,
            None => return Err(SplCalcErr::Ratio),
        };
        let pool_tokens_burnt = aaf.rem();
        let withdraw_lamports = match self.lst_to_lamports_ratio().apply(pool_tokens_burnt) {
            Some(w) => w,
            None => return Err(SplCalcErr::Ratio),
        };
        Ok(withdraw_lamports..=withdraw_lamports)
    }

    #[inline]
    pub const fn svc_sol_to_lst(
        &self,
        lamports_amount: u64,
    ) -> Result<RangeInclusive<u64>, SplCalcErr> {
        let r = match floor_ratio_u64_u64_reverse(self.lst_to_lamports_ratio(), lamports_amount) {
            Some(r) => r,
            None => return Err(SplCalcErr::Ratio),
        };
        let fee = match self.stake_withdrawal_fee_ceil() {
            Some(f) => f,
            None => return Err(SplCalcErr::Ratio),
        };
        let (min, max) = match (
            fee_ceil_ratio_u64_u64_reverse_from_rem(fee, *r.start()),
            fee_ceil_ratio_u64_u64_reverse_from_rem(fee, *r.end()),
        ) {
            (Some(min), Some(max)) => (*min.start(), *max.end()),
            _ => return Err(SplCalcErr::Ratio),
        };
        Ok(min..=max)
    }
}

impl SolValCalc for SplCalc {
    type Error = SplCalcErr;

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
pub enum SplCalcErr {
    Ratio,
    NotUpdated,
}

impl Display for SplCalcErr {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::NotUpdated => "not yet updated this epoch",
            Self::Ratio => "ratio math error",
        })
    }
}

impl Error for SplCalcErr {}
