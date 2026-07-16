use inf1_pp_core::{
    instructions::price::{exact_in::PriceExactInIxArgs, exact_out::PriceExactOutIxArgs},
    traits::main::{PriceExactIn, PriceExactOut},
};
use sanctum_u64_ratio::{Floor, Ratio};

#[allow(deprecated)]
use inf1_pp_core::{
    instructions::deprecated::lp::{
        mint::PriceLpTokensToMintIxArgs, redeem::PriceLpTokensToRedeemIxArgs,
    },
    traits::deprecated::{PriceLpTokensToMint, PriceLpTokensToRedeem},
};

use crate::{
    errs::ReserveV2ProgramErr,
    typedefs::{FeeEntryPacked, FeeNanos, ThresholdNanos, NANOS_DENOM},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FlatPricing {
    pub input_fee_nanos: FeeNanos,
    pub output_fee_nanos: FeeNanos,
}

impl FlatPricing {
    #[inline]
    pub const fn from_entries(input_entry: &FeeEntryPacked, output_entry: &FeeEntryPacked) -> Self {
        Self {
            input_fee_nanos: input_entry.base_fee_nanos(),
            output_fee_nanos: output_entry.output_fee_nanos(),
        }
    }

    #[inline]
    pub fn pp_price_exact_in(&self, input_sol_value: u64) -> Result<u64, ReserveV2ProgramErr> {
        price_exact_in_retained_product(
            input_sol_value,
            self.input_fee_nanos,
            self.output_fee_nanos,
        )
    }

    #[inline]
    pub fn pp_price_exact_out(&self, output_sol_value: u64) -> Result<u64, ReserveV2ProgramErr> {
        price_exact_out_retained_product(
            output_sol_value,
            self.input_fee_nanos,
            self.output_fee_nanos,
        )
    }
}

impl PriceExactIn for FlatPricing {
    type Error = ReserveV2ProgramErr;

    #[inline]
    fn price_exact_in(
        &self,
        PriceExactInIxArgs { sol_value, .. }: PriceExactInIxArgs,
    ) -> Result<u64, Self::Error> {
        self.pp_price_exact_in(sol_value)
    }
}

impl PriceExactOut for FlatPricing {
    type Error = ReserveV2ProgramErr;

    #[inline]
    fn price_exact_out(
        &self,
        PriceExactOutIxArgs { sol_value, .. }: PriceExactOutIxArgs,
    ) -> Result<u64, Self::Error> {
        self.pp_price_exact_out(sol_value)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct InputFeeCurve {
    pub base_fee_nanos: FeeNanos,
    pub threshold_nanos: ThresholdNanos,
    pub threshold_fee_nanos: FeeNanos,
    pub max_fee_nanos: FeeNanos,
}

impl InputFeeCurve {
    #[inline]
    pub const fn from_entry(entry: &FeeEntryPacked) -> Self {
        Self {
            base_fee_nanos: entry.base_fee_nanos(),
            threshold_nanos: entry.threshold_nanos(),
            threshold_fee_nanos: entry.threshold_fee_nanos(),
            max_fee_nanos: entry.max_fee_nanos(),
        }
    }

    #[inline]
    pub fn spot_fee_nanos(
        &self,
        pool_sol_value: u64,
        wsol_balance: u64,
    ) -> Result<FeeNanos, ReserveV2ProgramErr> {
        // if pool accounting is stale, actual wSOL holdings are a lower bound on pool value
        let effective_pool_sol_value = pool_sol_value.max(wsol_balance);
        if effective_pool_sol_value == 0 {
            return Err(ReserveV2ProgramErr::ZeroPoolSolValue);
        }
        let used = effective_pool_sol_value - wsol_balance;
        let threshold_nanos = self.threshold_nanos.get();

        // compare `used / pool <= threshold / N` by cross-multiplication
        let threshold_used_num = (effective_pool_sol_value as u128)
            .checked_mul(threshold_nanos as u128)
            .ok_or(ReserveV2ProgramErr::MathOverflow)?;
        let used_num = (used as u128)
            .checked_mul(NANOS_DENOM as u128)
            .ok_or(ReserveV2ProgramErr::MathOverflow)?;

        if used_num <= threshold_used_num {
            let delta = self
                .threshold_fee_nanos
                .get()
                .checked_sub(self.base_fee_nanos.get())
                .ok_or(ReserveV2ProgramErr::MathOverflow)?;
            // denominator here is `effective_pool_sol_value * threshold_nanos`, reuse `threshold_used_num`
            let extra = ceil_mul_div(delta as u128, used_num, threshold_used_num)?;
            return add_fee_nanos(self.base_fee_nanos, extra);
        }

        let delta = self
            .max_fee_nanos
            .get()
            .checked_sub(self.threshold_fee_nanos.get())
            .ok_or(ReserveV2ProgramErr::MathOverflow)?;
        let denom = (effective_pool_sol_value as u128)
            .checked_mul((NANOS_DENOM - threshold_nanos) as u128)
            .ok_or(ReserveV2ProgramErr::MathOverflow)?;
        let numerator = used_num
            .checked_sub(threshold_used_num)
            .ok_or(ReserveV2ProgramErr::MathOverflow)?;
        let extra = ceil_mul_div(delta as u128, numerator, denom)?;
        add_fee_nanos(self.threshold_fee_nanos, extra)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LpOutPricing {
    /// Input mint curve used at current wSOL utilization.
    pub input_fee_curve: InputFeeCurve,

    /// LP mint output fee.
    pub output_fee_nanos: FeeNanos,

    pub pool_sol_value: u64,
    pub wsol_balance: u64,
}

impl LpOutPricing {
    #[inline]
    pub const fn from_entries(
        input_entry: &FeeEntryPacked,
        output_entry: &FeeEntryPacked,
        pool_sol_value: u64,
        wsol_balance: u64,
    ) -> Self {
        Self {
            input_fee_curve: InputFeeCurve::from_entry(input_entry),
            output_fee_nanos: output_entry.output_fee_nanos(),
            pool_sol_value,
            wsol_balance,
        }
    }

    #[inline]
    pub fn spot_input_fee_nanos(&self) -> Result<FeeNanos, ReserveV2ProgramErr> {
        self.input_fee_curve
            .spot_fee_nanos(self.pool_sol_value, self.wsol_balance)
    }

    #[inline]
    pub fn pp_price_exact_in(&self, input_sol_value: u64) -> Result<u64, ReserveV2ProgramErr> {
        price_exact_in_retained_product(
            input_sol_value,
            self.spot_input_fee_nanos()?,
            self.output_fee_nanos,
        )
    }

    #[inline]
    pub fn pp_price_exact_out(&self, output_sol_value: u64) -> Result<u64, ReserveV2ProgramErr> {
        price_exact_out_retained_product(
            output_sol_value,
            self.spot_input_fee_nanos()?,
            self.output_fee_nanos,
        )
    }
}

impl PriceExactIn for LpOutPricing {
    type Error = ReserveV2ProgramErr;

    #[inline]
    fn price_exact_in(
        &self,
        PriceExactInIxArgs { sol_value, .. }: PriceExactInIxArgs,
    ) -> Result<u64, Self::Error> {
        self.pp_price_exact_in(sol_value)
    }
}

impl PriceExactOut for LpOutPricing {
    type Error = ReserveV2ProgramErr;

    #[inline]
    fn price_exact_out(
        &self,
        PriceExactOutIxArgs { sol_value, .. }: PriceExactOutIxArgs,
    ) -> Result<u64, Self::Error> {
        self.pp_price_exact_out(sol_value)
    }
}

#[allow(deprecated)]
impl PriceLpTokensToMint for FlatPricing {
    type Error = ReserveV2ProgramErr;

    #[inline]
    fn price_lp_tokens_to_mint(
        &self,
        _input: PriceLpTokensToMintIxArgs,
    ) -> Result<u64, Self::Error> {
        Err(ReserveV2ProgramErr::UnsupportedDeprecatedInstruction)
    }
}

#[allow(deprecated)]
impl PriceLpTokensToRedeem for FlatPricing {
    type Error = ReserveV2ProgramErr;

    #[inline]
    fn price_lp_tokens_to_redeem(
        &self,
        _input: PriceLpTokensToRedeemIxArgs,
    ) -> Result<u64, Self::Error> {
        Err(ReserveV2ProgramErr::UnsupportedDeprecatedInstruction)
    }
}

#[allow(deprecated)]
impl PriceLpTokensToMint for LpOutPricing {
    type Error = ReserveV2ProgramErr;

    #[inline]
    fn price_lp_tokens_to_mint(
        &self,
        _input: PriceLpTokensToMintIxArgs,
    ) -> Result<u64, Self::Error> {
        Err(ReserveV2ProgramErr::UnsupportedDeprecatedInstruction)
    }
}

#[allow(deprecated)]
impl PriceLpTokensToRedeem for LpOutPricing {
    type Error = ReserveV2ProgramErr;

    #[inline]
    fn price_lp_tokens_to_redeem(
        &self,
        _input: PriceLpTokensToRedeemIxArgs,
    ) -> Result<u64, Self::Error> {
        Err(ReserveV2ProgramErr::UnsupportedDeprecatedInstruction)
    }
}

#[inline]
fn price_exact_in_retained_product(
    input_sol_value: u64,
    input_fee_nanos: FeeNanos,
    output_fee_nanos: FeeNanos,
) -> Result<u64, ReserveV2ProgramErr> {
    let ratio = route_retained_ratio(input_fee_nanos, output_fee_nanos)?;
    ratio
        .apply(input_sol_value)
        .ok_or(ReserveV2ProgramErr::MathOverflow)
}

#[inline]
fn price_exact_out_retained_product(
    output_sol_value: u64,
    input_fee_nanos: FeeNanos,
    output_fee_nanos: FeeNanos,
) -> Result<u64, ReserveV2ProgramErr> {
    let ratio = route_retained_ratio(input_fee_nanos, output_fee_nanos)?;
    let range = ratio
        .reverse(output_sol_value)
        .ok_or(ReserveV2ProgramErr::MathOverflow)?;
    Ok(*range.start())
}

#[inline]
fn route_retained_ratio(
    input_fee_nanos: FeeNanos,
    output_fee_nanos: FeeNanos,
) -> Result<Floor<Ratio<u64, u64>>, ReserveV2ProgramErr> {
    let input_retained_nanos = input_fee_nanos.retained() as u64;
    let output_retained_nanos = output_fee_nanos.retained() as u64;
    // retained values are <= NANOS_DENOM so product <= N^2, which fits in u64
    let retained_product = input_retained_nanos * output_retained_nanos;
    if retained_product == 0 {
        return Err(ReserveV2ProgramErr::ZeroRetainedValue);
    }
    let denom = (NANOS_DENOM as u64) * (NANOS_DENOM as u64);
    Ok(Floor(Ratio {
        n: retained_product,
        d: denom,
    }))
}

#[inline]
fn ceil_mul_div(a: u128, b: u128, denominator: u128) -> Result<u128, ReserveV2ProgramErr> {
    if denominator == 0 {
        return Err(ReserveV2ProgramErr::MathOverflow);
    }
    Ok(a.checked_mul(b)
        .ok_or(ReserveV2ProgramErr::MathOverflow)?
        .div_ceil(denominator))
}

#[inline]
fn add_fee_nanos(fee_nanos: FeeNanos, extra: u128) -> Result<FeeNanos, ReserveV2ProgramErr> {
    if extra > fee_nanos.retained() as u128 {
        return Err(ReserveV2ProgramErr::MathOverflow);
    }
    FeeNanos::new(fee_nanos.get() + extra as u32).map_err(|_| ReserveV2ProgramErr::MathOverflow)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flat_pricing_basic() {
        let pricing = FlatPricing {
            input_fee_nanos: FeeNanos::new(500_000_000).unwrap(),
            output_fee_nanos: FeeNanos::new(600_000_000).unwrap(),
        };
        let amt = 0;
        let sol_value = 1_000;
        let exact_in = pricing
            .price_exact_in(PriceExactInIxArgs { amt, sol_value })
            .unwrap();
        assert_eq!(exact_in, 200);
        let exact_out = pricing
            .price_exact_out(PriceExactOutIxArgs { amt, sol_value })
            .unwrap();
        assert_eq!(exact_out, 5_000);
    }

    #[test]
    fn flat_pricing_exact_out_rounds_up() {
        // 1bps
        let pricing = FlatPricing {
            input_fee_nanos: FeeNanos::new(100_000).unwrap(),
            output_fee_nanos: FeeNanos::ZERO,
        };
        let amt = 0;
        let exact_in = pricing
            .price_exact_in(PriceExactInIxArgs {
                amt,
                sol_value: 999_999_999,
            })
            .unwrap();
        assert_eq!(exact_in, 999_899_999);
        // non-exact division: minimal sufficient input rounds up
        let exact_out = pricing
            .price_exact_out(PriceExactOutIxArgs {
                amt,
                sol_value: 999_899_999,
            })
            .unwrap();
        assert_eq!(exact_out, 999_999_999);
        // exact division: no rounding
        let exact_out = pricing
            .price_exact_out(PriceExactOutIxArgs {
                amt,
                sol_value: 999_900_000,
            })
            .unwrap();
        assert_eq!(exact_out, 1_000_000_000);
    }

    #[test]
    fn flat_pricing_zero_retained_value() {
        for pricing in [
            FlatPricing {
                input_fee_nanos: FeeNanos::MAX,
                output_fee_nanos: FeeNanos::ZERO,
            },
            FlatPricing {
                input_fee_nanos: FeeNanos::ZERO,
                output_fee_nanos: FeeNanos::MAX,
            },
        ] {
            let amt = 0;
            let sol_value = 1;
            let exact_in = pricing.price_exact_in(PriceExactInIxArgs { amt, sol_value });
            assert_eq!(exact_in, Err(ReserveV2ProgramErr::ZeroRetainedValue));
            let exact_out = pricing.price_exact_out(PriceExactOutIxArgs { amt, sol_value });
            assert_eq!(exact_out, Err(ReserveV2ProgramErr::ZeroRetainedValue));
        }
    }

    #[test]
    fn input_fee_curve_spot_fee() {
        let curve = InputFeeCurve {
            base_fee_nanos: FeeNanos::new(100_000_000).unwrap(),
            threshold_nanos: ThresholdNanos::new(500_000_000).unwrap(),
            threshold_fee_nanos: FeeNanos::new(300_000_000).unwrap(),
            max_fee_nanos: FeeNanos::new(900_000_000).unwrap(),
        };
        assert_eq!(
            curve.spot_fee_nanos(100, 150).unwrap(),
            curve.base_fee_nanos
        );
        assert_eq!(
            curve.spot_fee_nanos(100, 75).unwrap(),
            FeeNanos::new(200_000_000).unwrap()
        );
        assert_eq!(
            curve.spot_fee_nanos(100, 50).unwrap(),
            curve.threshold_fee_nanos
        );
        assert_eq!(
            curve.spot_fee_nanos(100, 25).unwrap(),
            FeeNanos::new(600_000_000).unwrap()
        );
        assert_eq!(curve.spot_fee_nanos(100, 0).unwrap(), curve.max_fee_nanos);
    }

    #[test]
    fn input_fee_curve_spot_fee_zero_pool_value() {
        let curve = InputFeeCurve {
            base_fee_nanos: FeeNanos::ZERO,
            threshold_nanos: ThresholdNanos::MIN,
            threshold_fee_nanos: FeeNanos::ZERO,
            max_fee_nanos: FeeNanos::MAX,
        };
        assert_eq!(
            curve.spot_fee_nanos(0, 0),
            Err(ReserveV2ProgramErr::ZeroPoolSolValue)
        );
    }

    #[test]
    fn lp_out_basic() {
        let pricing = LpOutPricing {
            input_fee_curve: InputFeeCurve {
                base_fee_nanos: FeeNanos::new(100_000_000).unwrap(),
                threshold_nanos: ThresholdNanos::new(500_000_000).unwrap(),
                threshold_fee_nanos: FeeNanos::new(300_000_000).unwrap(),
                max_fee_nanos: FeeNanos::new(900_000_000).unwrap(),
            },
            output_fee_nanos: FeeNanos::new(500_000_000).unwrap(),
            pool_sol_value: 100,
            wsol_balance: 75,
        };
        let amt = 0;
        let sol_value = 1_000;

        assert_eq!(pricing.spot_input_fee_nanos().unwrap().get(), 200_000_000);

        let exact_in = pricing
            .price_exact_in(PriceExactInIxArgs { amt, sol_value })
            .unwrap();
        assert_eq!(exact_in, 400);
        let exact_out = pricing
            .price_exact_out(PriceExactOutIxArgs { amt, sol_value })
            .unwrap();
        assert_eq!(exact_out, 2_500);
    }

    #[test]
    fn lp_out_zero_pool_value() {
        let curve = InputFeeCurve {
            base_fee_nanos: FeeNanos::ZERO,
            threshold_nanos: ThresholdNanos::MIN,
            threshold_fee_nanos: FeeNanos::ZERO,
            max_fee_nanos: FeeNanos::MAX,
        };
        let pricing = LpOutPricing {
            input_fee_curve: curve,
            output_fee_nanos: FeeNanos::ZERO,
            pool_sol_value: 0,
            wsol_balance: 0,
        };
        let amt = 0;
        let sol_value = 1_000;
        let exact_in = pricing.price_exact_in(PriceExactInIxArgs { amt, sol_value });
        assert_eq!(exact_in, Err(ReserveV2ProgramErr::ZeroPoolSolValue));
    }

    #[test]
    #[allow(deprecated)]
    fn deprecated_lp_instructions() {
        let flat_pricing = FlatPricing {
            input_fee_nanos: FeeNanos::ZERO,
            output_fee_nanos: FeeNanos::ZERO,
        };
        let amt = 0;
        let sol_value = 1;
        let mint =
            flat_pricing.price_lp_tokens_to_mint(PriceLpTokensToMintIxArgs { amt, sol_value });
        assert_eq!(
            mint,
            Err(ReserveV2ProgramErr::UnsupportedDeprecatedInstruction)
        );
        let redeem =
            flat_pricing.price_lp_tokens_to_redeem(PriceLpTokensToRedeemIxArgs { amt, sol_value });
        assert_eq!(
            redeem,
            Err(ReserveV2ProgramErr::UnsupportedDeprecatedInstruction)
        );

        let lp_out_pricing = LpOutPricing {
            input_fee_curve: InputFeeCurve {
                base_fee_nanos: FeeNanos::ZERO,
                threshold_nanos: ThresholdNanos::MIN,
                threshold_fee_nanos: FeeNanos::ZERO,
                max_fee_nanos: FeeNanos::ZERO,
            },
            output_fee_nanos: FeeNanos::ZERO,
            pool_sol_value: 1,
            wsol_balance: 1,
        };
        let mint =
            lp_out_pricing.price_lp_tokens_to_mint(PriceLpTokensToMintIxArgs { amt, sol_value });
        assert_eq!(
            mint,
            Err(ReserveV2ProgramErr::UnsupportedDeprecatedInstruction)
        );
        let redeem = lp_out_pricing
            .price_lp_tokens_to_redeem(PriceLpTokensToRedeemIxArgs { amt, sol_value });
        assert_eq!(
            redeem,
            Err(ReserveV2ProgramErr::UnsupportedDeprecatedInstruction)
        );
    }
}
