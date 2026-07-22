use inf1_pp_core::{
    instructions::price::exact_out::PriceExactOutIxArgs, traits::main::PriceExactOut,
};
use sanctum_u64_ratio::{Floor, Ratio};

use crate::{
    errs::{OverCapErr, ReserveV2ProgramErr},
    typedefs::{FeeEntry, FeeNanos, ThresholdNanos, NANOS_DENOM},
};

use super::retained::price_exact_out_retained_product;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct InputFeeCurve {
    pub base_fee_nanos: FeeNanos,
    pub threshold_nanos: ThresholdNanos,
    pub threshold_fee_nanos: FeeNanos,
    pub max_fee_nanos: FeeNanos,
}

impl InputFeeCurve {
    #[inline]
    pub const fn from_entry(entry: &FeeEntry) -> Self {
        Self {
            base_fee_nanos: entry.nanos.base_fee_nanos(),
            threshold_nanos: entry.nanos.threshold_nanos(),
            threshold_fee_nanos: entry.nanos.threshold_fee_nanos(),
            max_fee_nanos: entry.nanos.max_fee_nanos(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RangeOutPricing {
    pub input_fee_curve: InputFeeCurve,
    pub output_fee_nanos: FeeNanos,
    pub pool_sol_value: u64,
    pub wsol_balance: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct RangeOutState {
    effective_pool_sol_value: u64,
    used_before: u64,
    threshold_lamports: u64,
}

impl RangeOutPricing {
    #[inline]
    pub const fn from_entries(
        input_entry: &FeeEntry,
        output_entry: &FeeEntry,
        pool_sol_value: u64,
        wsol_balance: u64,
    ) -> Self {
        Self {
            input_fee_curve: InputFeeCurve::from_entry(input_entry),
            output_fee_nanos: output_entry.nanos.output_fee_nanos(),
            pool_sol_value,
            wsol_balance,
        }
    }

    #[inline]
    pub fn pp_price_exact_out(&self, output_sol_value: u64) -> Result<u64, ReserveV2ProgramErr> {
        if output_sol_value > self.wsol_balance {
            return Err(ReserveV2ProgramErr::OverCap(OverCapErr {
                requested_out_sol_value: output_sol_value,
                wsol_balance: self.wsol_balance,
            }));
        }

        let RangeOutState {
            effective_pool_sol_value,
            used_before,
            threshold_lamports,
        } = self.range_out_state()?;
        let used_after = used_before
            .checked_add(output_sol_value)
            .ok_or(ReserveV2ProgramErr::MathOverflow)?;

        let mut required_input = 0u64;
        let mut used_cursor = used_before;

        if used_cursor < threshold_lamports {
            let band = Band {
                start_used: 0,
                end_used: threshold_lamports,
                start_fee_nanos: self.input_fee_curve.base_fee_nanos,
                end_fee_nanos: self.input_fee_curve.threshold_fee_nanos,
            };
            let piece_end = used_after.min(threshold_lamports);
            required_input = required_input
                .checked_add(self.price_exact_out_piece(used_cursor, piece_end, band)?)
                .ok_or(ReserveV2ProgramErr::MathOverflow)?;
            used_cursor = piece_end;
        }

        if used_cursor < used_after {
            let band = Band {
                start_used: threshold_lamports,
                end_used: effective_pool_sol_value,
                start_fee_nanos: self.input_fee_curve.threshold_fee_nanos,
                end_fee_nanos: self.input_fee_curve.max_fee_nanos,
            };
            required_input = required_input
                .checked_add(self.price_exact_out_piece(used_cursor, used_after, band)?)
                .ok_or(ReserveV2ProgramErr::MathOverflow)?;
        }

        Ok(required_input)
    }

    #[inline]
    fn range_out_state(&self) -> Result<RangeOutState, ReserveV2ProgramErr> {
        let effective_pool_sol_value = self.pool_sol_value.max(self.wsol_balance);
        if effective_pool_sol_value == 0 {
            return Err(ReserveV2ProgramErr::ZeroPoolSolValue);
        }
        let used_before = effective_pool_sol_value - self.wsol_balance;
        let threshold_lamports = Floor(Ratio {
            n: self.input_fee_curve.threshold_nanos.get(),
            d: NANOS_DENOM,
        })
        .apply(effective_pool_sol_value)
        .ok_or(ReserveV2ProgramErr::MathOverflow)?;

        Ok(RangeOutState {
            effective_pool_sol_value,
            used_before,
            threshold_lamports,
        })
    }

    #[inline]
    fn price_exact_out_piece(
        &self,
        piece_start_used: u64,
        piece_end_used: u64,
        band: Band,
    ) -> Result<u64, ReserveV2ProgramErr> {
        let piece_output = piece_end_used
            .checked_sub(piece_start_used)
            .ok_or(ReserveV2ProgramErr::MathOverflow)?;
        if piece_output == 0 {
            return Ok(0);
        }

        let band_width = match band.end_used.checked_sub(band.start_used) {
            Some(width) if width > 0 => width,
            _ => return Err(ReserveV2ProgramErr::MathOverflow),
        };
        let delta = band
            .end_fee_nanos
            .get()
            .checked_sub(band.start_fee_nanos.get())
            .ok_or(ReserveV2ProgramErr::MathOverflow)?;
        let offset_start = piece_start_used
            .checked_sub(band.start_used)
            .ok_or(ReserveV2ProgramErr::MathOverflow)?;
        let offset_end = piece_end_used
            .checked_sub(band.start_used)
            .ok_or(ReserveV2ProgramErr::MathOverflow)?;
        let midpoint_offset_sum = u128::from(offset_start)
            .checked_add(u128::from(offset_end))
            .ok_or(ReserveV2ProgramErr::MathOverflow)?;
        let denom = u128::from(band_width)
            .checked_mul(2)
            .ok_or(ReserveV2ProgramErr::MathOverflow)?;
        let extra = ceil_mul_div(u128::from(delta), midpoint_offset_sum, denom)?;
        let piece_input_fee_nanos = add_fee_nanos(band.start_fee_nanos, extra)?;

        price_exact_out_retained_product(piece_output, piece_input_fee_nanos, self.output_fee_nanos)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Band {
    start_used: u64,
    end_used: u64,
    start_fee_nanos: FeeNanos,
    end_fee_nanos: FeeNanos,
}

impl PriceExactOut for RangeOutPricing {
    type Error = ReserveV2ProgramErr;

    #[inline]
    fn price_exact_out(
        &self,
        PriceExactOutIxArgs { sol_value, .. }: PriceExactOutIxArgs,
    ) -> Result<u64, Self::Error> {
        self.pp_price_exact_out(sol_value)
    }
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
    if extra > u128::from(fee_nanos.retained().get()) {
        return Err(ReserveV2ProgramErr::MathOverflow);
    }
    let extra: u32 = extra
        .try_into()
        .map_err(|_| ReserveV2ProgramErr::MathOverflow)?;
    FeeNanos::new(fee_nanos.get() + extra).map_err(|_| ReserveV2ProgramErr::MathOverflow)
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    const LAMPORTS_PER_SOL: u64 = 1_000_000_000;
    const TEST_POOL_SOL_VALUE: u64 = 1_000_000 * LAMPORTS_PER_SOL;
    const TEST_BASE_FEE_NANOS: u32 = 0;
    const TEST_THRESHOLD_NANOS: u32 = 200_000_000; // 20%
    const TEST_THRESHOLD_FEE_NANOS: u32 = 10_000_000; // 1%
    const TEST_MAX_FEE_NANOS: u32 = 100_000_000; // 10%

    fn range_out_pricing(pool_sol_value: u64, wsol_balance: u64) -> RangeOutPricing {
        RangeOutPricing {
            input_fee_curve: InputFeeCurve {
                base_fee_nanos: FeeNanos::new(TEST_BASE_FEE_NANOS).unwrap(),
                threshold_nanos: ThresholdNanos::new(TEST_THRESHOLD_NANOS).unwrap(),
                threshold_fee_nanos: FeeNanos::new(TEST_THRESHOLD_FEE_NANOS).unwrap(),
                max_fee_nanos: FeeNanos::new(TEST_MAX_FEE_NANOS).unwrap(),
            },
            output_fee_nanos: FeeNanos::ZERO,
            pool_sol_value,
            wsol_balance,
        }
    }

    fn price_exact_out(
        pricing: &RangeOutPricing,
        sol_value: u64,
    ) -> Result<u64, ReserveV2ProgramErr> {
        pricing.price_exact_out(PriceExactOutIxArgs { amt: 0, sol_value })
    }

    #[test]
    fn exact_out_band_1() {
        let pricing = range_out_pricing(TEST_POOL_SOL_VALUE, TEST_POOL_SOL_VALUE);

        // 0 -> 100k SOL, midpoint fee = 0.25%
        assert_eq!(
            price_exact_out(&pricing, 100_000 * LAMPORTS_PER_SOL),
            Ok(100_250_626_566_417)
        );
    }

    #[test]
    fn exact_out_crosses_threshold() {
        let pricing = range_out_pricing(TEST_POOL_SOL_VALUE, 850_000 * LAMPORTS_PER_SOL);

        // 150k -> threshold = 200k SOL, then threshold -> 250k SOL
        assert_eq!(
            price_exact_out(&pricing, 100_000 * LAMPORTS_PER_SOL),
            Ok(101_090_301_454_601)
        );
    }

    #[test]
    fn exact_out_band_2() {
        let pricing = range_out_pricing(TEST_POOL_SOL_VALUE, 450_000 * LAMPORTS_PER_SOL);

        // 550k -> 650k SOL, midpoint fee = 5.5%
        assert_eq!(
            price_exact_out(&pricing, 100_000 * LAMPORTS_PER_SOL),
            Ok(105_820_105_820_106)
        );
    }

    #[test]
    fn exact_out_over_cap() {
        let wsol_balance = 10 * LAMPORTS_PER_SOL;
        let requested_out_sol_value = wsol_balance + 1;
        let pricing = range_out_pricing(TEST_POOL_SOL_VALUE, wsol_balance);

        assert_eq!(
            price_exact_out(&pricing, requested_out_sol_value),
            Err(ReserveV2ProgramErr::OverCap(OverCapErr {
                requested_out_sol_value,
                wsol_balance,
            }))
        );
    }

    #[test]
    fn exact_out_zero_retained_value() {
        let pricing = RangeOutPricing {
            input_fee_curve: InputFeeCurve {
                base_fee_nanos: FeeNanos::ZERO,
                threshold_nanos: ThresholdNanos::new(TEST_THRESHOLD_NANOS).unwrap(),
                threshold_fee_nanos: FeeNanos::MAX,
                max_fee_nanos: FeeNanos::MAX,
            },
            output_fee_nanos: FeeNanos::ZERO,
            pool_sol_value: TEST_POOL_SOL_VALUE,
            wsol_balance: 650_000 * LAMPORTS_PER_SOL,
        };

        assert_eq!(
            price_exact_out(&pricing, 1),
            Err(ReserveV2ProgramErr::ZeroRetainedValue)
        );
    }

    #[test]
    fn exact_out_over_liquid_equals_synced_state() {
        let over_liquid = range_out_pricing(TEST_POOL_SOL_VALUE, 1_050_000 * LAMPORTS_PER_SOL);
        let synced = range_out_pricing(1_050_000 * LAMPORTS_PER_SOL, 1_050_000 * LAMPORTS_PER_SOL);

        assert_eq!(
            price_exact_out(&over_liquid, 10_000 * LAMPORTS_PER_SOL),
            price_exact_out(&synced, 10_000 * LAMPORTS_PER_SOL)
        );
    }

    // proptests

    fn fee_nanos_for_props() -> impl Strategy<Value = FeeNanos> {
        (0..=NANOS_DENOM).prop_map(|fee_nanos| FeeNanos::new(fee_nanos).unwrap())
    }

    fn input_fee_curve_for_props() -> impl Strategy<Value = InputFeeCurve> {
        (
            1..NANOS_DENOM,
            0..=NANOS_DENOM,
            0..=NANOS_DENOM,
            0..=NANOS_DENOM,
        )
            .prop_map(|(threshold_nanos, fee_nanos_a, fee_nanos_b, fee_nanos_c)| {
                let mut fee_nanos = [fee_nanos_a, fee_nanos_b, fee_nanos_c];
                fee_nanos.sort();
                InputFeeCurve {
                    base_fee_nanos: FeeNanos::new(fee_nanos[0]).unwrap(),
                    threshold_nanos: ThresholdNanos::new(threshold_nanos).unwrap(),
                    threshold_fee_nanos: FeeNanos::new(fee_nanos[1]).unwrap(),
                    max_fee_nanos: FeeNanos::new(fee_nanos[2]).unwrap(),
                }
            })
    }

    prop_compose! {
        fn range_out_pricing_props()
            (
                input_fee_curve in input_fee_curve_for_props(),
                output_fee_nanos in fee_nanos_for_props(),
                pool_sol_value in 1..=u64::MAX,
                wsol_balance: u64,
            ) -> RangeOutPricing {
                RangeOutPricing {
                    input_fee_curve,
                    output_fee_nanos,
                    pool_sol_value,
                    wsol_balance,
                }
            }
    }

    proptest! {
        #[test]
        fn exact_out_gte_requested_output(
            pricing in range_out_pricing_props(),
            output_sol_value: u64,
        ) {
            let output_sol_value = output_sol_value.min(pricing.wsol_balance);
            match price_exact_out(&pricing, output_sol_value) {
                Ok(required_input) => prop_assert!(required_input >= output_sol_value),
                Err(err) => prop_assert!(matches!(
                    err,
                    ReserveV2ProgramErr::MathOverflow | ReserveV2ProgramErr::ZeroRetainedValue
                )),
            }
        }
    }
}
