use inf1_pp_core::{
    instructions::price::{exact_in::PriceExactInIxArgs, exact_out::PriceExactOutIxArgs},
    traits::main::{PriceExactIn, PriceExactOut},
};
use sanctum_token_ratio_compat::floor_ratio_u64_u64_reverse;
use sanctum_u64_ratio::{Floor, Ratio};

use super::err::FlatFeePricingErr;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FlatFeeSwapPricing {
    /// Read from [`crate::accounts::fee::FeeAccount`] of input LST
    pub input_fee_bps: i16,

    /// Read from [`crate::accounts::fee::FeeAccount`] of output LST
    pub output_fee_bps: i16,
}

type Fr = Floor<Ratio<u16, u16>>;

impl FlatFeeSwapPricing {
    /// Returns the ratio that returns out_sol_value
    /// when applied to in_sol_value
    ///
    /// Returns None if self's data result in overflow
    #[inline]
    pub const fn out_ratio(&self) -> Option<Fr> {
        let fee_bps = match self.input_fee_bps.checked_add(self.output_fee_bps) {
            None => return None,
            Some(f) => f,
        };
        // post_fee_bps = 10_000 - fee_bps
        // out_sol_value = floor(in_sol_value * post_fee_bps / 10_000)
        // i16 signed subtraction:
        // - rebates are not allowed (post_fee_bps > 10_000)
        // - >100% fees will error (post_fee_bps < 0)
        let post_fee_bps = match 10_000i16.checked_sub(fee_bps) {
            None => return None,
            Some(f) => f,
        };
        let post_fee_bps = if post_fee_bps < 0 {
            return None;
        } else {
            post_fee_bps as u16
        };
        Some(Floor(Ratio {
            n: post_fee_bps,
            d: 10_000,
        }))
    }

    #[inline]
    pub const fn pp_price_exact_in(&self, in_sol_value: u64) -> Option<u64> {
        match self.out_ratio() {
            None => None,
            Some(r) => r.apply(in_sol_value),
        }
    }

    #[inline]
    pub const fn pp_price_exact_out(&self, out_sol_value: u64) -> Option<u64> {
        let range_opt = match self.out_ratio() {
            None => return None,
            Some(Floor(Ratio { n, d })) => floor_ratio_u64_u64_reverse(
                Floor(Ratio {
                    // as-safety: smaller bitwidth as larger bitwidth
                    n: n as u64,
                    d: d as u64,
                }),
                out_sol_value,
            ),
        };
        let range = match range_opt {
            None => return None,
            Some(r) => r,
        };
        Some(*range.end())
    }
}

impl PriceExactIn for FlatFeeSwapPricing {
    type Error = FlatFeePricingErr;

    #[inline]
    fn price_exact_in(
        &self,
        PriceExactInIxArgs { sol_value, .. }: PriceExactInIxArgs,
    ) -> Result<u64, Self::Error> {
        self.pp_price_exact_in(sol_value)
            .ok_or(FlatFeePricingErr::Ratio)
    }
}

impl PriceExactOut for FlatFeeSwapPricing {
    type Error = FlatFeePricingErr;

    #[inline]
    fn price_exact_out(
        &self,
        PriceExactOutIxArgs { sol_value, .. }: PriceExactOutIxArgs,
    ) -> Result<u64, Self::Error> {
        self.pp_price_exact_out(sol_value)
            .ok_or(FlatFeePricingErr::Ratio)
    }
}
