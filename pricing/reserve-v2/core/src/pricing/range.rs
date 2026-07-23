use inf1_pp_core::{
    instructions::price::{exact_in::PriceExactInIxArgs, exact_out::PriceExactOutIxArgs},
    traits::main::{PriceExactIn, PriceExactOut},
};

#[allow(deprecated)]
use inf1_pp_core::{
    instructions::deprecated::lp::{
        mint::PriceLpTokensToMintIxArgs, redeem::PriceLpTokensToRedeemIxArgs,
    },
    traits::deprecated::{PriceLpTokensToMint, PriceLpTokensToRedeem},
};
use sanctum_u64_ratio::Floor;

use crate::{
    errs::{
        ExactInForwardCheckFailedErr, OverCapErr, ReserveV2ProgramErr, WsolBalanceGtPoolSolValueErr,
    },
    typedefs::{FeeEntry, FeeNanos, ThresholdNanos, NANOS_DENOM},
};

use super::retained::{price_exact_in_retained_product, price_exact_out_retained_product};

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
    pool_sol_value: u64,
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
    pub fn pp_price_exact_in(&self, input_sol_value: u64) -> Result<u64, ReserveV2ProgramErr> {
        if input_sol_value == 0 {
            return Ok(0);
        }

        let RangeOutState {
            pool_sol_value,
            used_before,
            threshold_lamports,
        } = self.range_out_state()?;

        let mut input_left = input_sol_value;
        let mut output_sol_value = 0u64;
        let mut used_cursor = used_before;

        if used_cursor < threshold_lamports {
            let band = Band {
                start_used: 0,
                end_used: threshold_lamports,
                start_fee_nanos: self.input_fee_curve.base_fee_nanos,
                end_fee_nanos: self.input_fee_curve.threshold_fee_nanos,
            };
            match self.consume_exact_in_piece(input_left, used_cursor, band.end_used, band)? {
                ExactInPiece::Full { output, input_used } => {
                    output_sol_value = output_sol_value
                        .checked_add(output)
                        .ok_or(ReserveV2ProgramErr::MathOverflow)?;
                    input_left = input_left
                        .checked_sub(input_used)
                        .ok_or(ReserveV2ProgramErr::MathOverflow)?;
                    used_cursor = band.end_used;
                }
                ExactInPiece::Partial { output } => {
                    output_sol_value = output_sol_value
                        .checked_add(output)
                        .ok_or(ReserveV2ProgramErr::MathOverflow)?;
                    return self.checked_exact_in_output(output_sol_value, input_sol_value);
                }
            }
        }

        if used_cursor < pool_sol_value {
            let band = Band {
                start_used: threshold_lamports,
                end_used: pool_sol_value,
                start_fee_nanos: self.input_fee_curve.threshold_fee_nanos,
                end_fee_nanos: self.input_fee_curve.max_fee_nanos,
            };
            match self.consume_exact_in_piece(input_left, used_cursor, band.end_used, band)? {
                ExactInPiece::Full { output, input_used } => {
                    output_sol_value = output_sol_value
                        .checked_add(output)
                        .ok_or(ReserveV2ProgramErr::MathOverflow)?;
                    input_left = input_left
                        .checked_sub(input_used)
                        .ok_or(ReserveV2ProgramErr::MathOverflow)?;
                }
                ExactInPiece::Partial { output } => {
                    output_sol_value = output_sol_value
                        .checked_add(output)
                        .ok_or(ReserveV2ProgramErr::MathOverflow)?;
                    return self.checked_exact_in_output(output_sol_value, input_sol_value);
                }
            }
        }

        if input_left > 0 {
            return Err(ReserveV2ProgramErr::OverCap(OverCapErr {
                requested_out_sol_value: self.wsol_balance.saturating_add(1),
                wsol_balance: self.wsol_balance,
            }));
        }

        self.checked_exact_in_output(output_sol_value, input_sol_value)
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
            pool_sol_value,
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
                end_used: pool_sol_value,
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
        if self.pool_sol_value == 0 {
            return Err(ReserveV2ProgramErr::ZeroPoolSolValue);
        }
        if self.wsol_balance > self.pool_sol_value {
            return Err(ReserveV2ProgramErr::WsolBalanceGtPoolSolValue(
                WsolBalanceGtPoolSolValueErr {
                    pool_sol_value: self.pool_sol_value,
                    wsol_balance: self.wsol_balance,
                },
            ));
        }
        let used_before = self.pool_sol_value - self.wsol_balance;
        let threshold_lamports = Floor(self.input_fee_curve.threshold_nanos.ratio())
            .apply(self.pool_sol_value)
            .ok_or(ReserveV2ProgramErr::MathOverflow)?;

        Ok(RangeOutState {
            pool_sol_value: self.pool_sol_value,
            used_before,
            threshold_lamports,
        })
    }

    #[inline]
    fn checked_exact_in_output(
        &self,
        output_sol_value: u64,
        input_sol_value: u64,
    ) -> Result<u64, ReserveV2ProgramErr> {
        let required_input = self.pp_price_exact_out(output_sol_value)?;
        if required_input <= input_sol_value {
            return Ok(output_sol_value);
        }

        Err(ReserveV2ProgramErr::ExactInForwardCheckFailed(
            ExactInForwardCheckFailedErr {
                input_sol_value,
                output_sol_value,
                required_input_sol_value: required_input,
            },
        ))
    }

    #[inline]
    fn consume_exact_in_piece(
        &self,
        input_left: u64,
        piece_start_used: u64,
        piece_end_used: u64,
        band: Band,
    ) -> Result<ExactInPiece, ReserveV2ProgramErr> {
        let full_piece_output = piece_end_used
            .checked_sub(piece_start_used)
            .ok_or(ReserveV2ProgramErr::MathOverflow)?;
        // Full-piece exact-out can overflow or round to zero retained value at
        // the piece end while a smaller partial exact-in quote is still valid.
        // Try partial solving instead of rejecting the whole quote.
        if let Ok(full_piece_cost) =
            self.price_exact_out_piece(piece_start_used, piece_end_used, band)
        {
            if full_piece_cost <= input_left {
                return Ok(ExactInPiece::Full {
                    output: full_piece_output,
                    input_used: full_piece_cost,
                });
            }
        }

        self.price_exact_in_partial_piece(input_left, full_piece_output, piece_start_used, band)
            .map(|output| ExactInPiece::Partial { output })
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

        let band_width = band.width()?;
        let delta = band.delta()?;
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

    #[inline]
    fn price_exact_in_partial_piece(
        &self,
        input_left: u64,
        max_piece_output: u64,
        piece_start_used: u64,
        band: Band,
    ) -> Result<u64, ReserveV2ProgramErr> {
        if input_left == 0 {
            return Ok(0);
        }

        if max_piece_output == 0 {
            return Ok(0);
        }

        let band_width = band.width()?;
        let delta = band.delta()?;
        let offset_start = piece_start_used
            .checked_sub(band.start_used)
            .ok_or(ReserveV2ProgramErr::MathOverflow)?;
        let entry_extra = ceil_mul_div(
            u128::from(delta),
            u128::from(offset_start),
            u128::from(band_width),
        )?;
        let entry_fee_nanos = add_fee_nanos(band.start_fee_nanos, entry_extra)?;

        let output_retained_nanos = u128::from(self.output_fee_nanos.retained().get());
        let real_entry_retained_nanos = entry_fee_nanos.retained().get();
        if output_retained_nanos == 0 || real_entry_retained_nanos == 0 {
            return Err(ReserveV2ProgramErr::ZeroRetainedValue);
        }
        if delta == 0 {
            let output = price_exact_in_retained_product(
                input_left,
                entry_fee_nanos,
                self.output_fee_nanos,
            )?;
            if output > max_piece_output {
                return Err(ReserveV2ProgramErr::MathOverflow);
            }
            return Ok(output);
        }
        // The +1 nano safety buffer below would turn a valid near-100% fee into zero retained value.
        // Return 0 output instead of a false ZeroRetainedValue error.
        if real_entry_retained_nanos == 1 {
            return Ok(0);
        }
        // `exact_in` computes the fee as `entry_fee + slope`, `exact_out`` uses the midpoint fee.
        // Algebraically they invert the same relation, but rounding (ceil on fees, floor on output)
        // results in them not being an exact inverse, so exact-in over-quotes and the pool overpays:
        // `exact_out(output) > input` for `output = exact_in(input)`
        // Round the fee up by 1 nano so exact-in under-quotes slightly and have exact-out
        // forward check (checked_exact_in_output) as a safety assertion.
        let safe_entry_fee_nanos = FeeNanos::new(
            entry_fee_nanos
                .get()
                .checked_add(1)
                .ok_or(ReserveV2ProgramErr::MathOverflow)?,
        )
        .map_err(|_| ReserveV2ProgramErr::MathOverflow)?;
        let entry_retained_nanos = u128::from(safe_entry_fee_nanos.retained().get());

        // Input amount and fee at the start of the piece is known, but the final output is unknown.
        //
        // Input fee increases linearly with output consumed:
        // slope = (band.end_fee - band.start_fee) / band_width
        //
        // Midpoint pricing makes the fee:
        // input_fee(output) = entry_fee + slope * output / 2
        //
        // This gives the circular equation:
        // output = input * output_retained * input_retained
        // output = input * output_retained * (1 - entry_fee - slope * output / 2)
        //
        // Rearranged:
        // output = input * output_retained * (1 - entry_fee)
        //          / (1 + input * output_retained * slope / 2)
        //
        // Substituting nanos-scaled fees, then multiplying numerator and denominator by N^2:
        // output = input * output_retained_nanos * entry_retained_nanos
        //          / (N^2 + input * output_retained_nanos * fee_delta_nanos / (2 * band_width))
        //
        // - `input_times_output_retained` is input * output_retained_nanos.
        // - `slope_term` is input * output_retained_nanos * fee_delta_nanos
        //   / (2 * band_width).
        let input_times_output_retained = u128::from(input_left)
            .checked_mul(output_retained_nanos)
            .ok_or(ReserveV2ProgramErr::MathOverflow)?;
        let slope_denom = u128::from(band_width)
            .checked_mul(2)
            .ok_or(ReserveV2ProgramErr::MathOverflow)?;
        let slope_term = ceil_mul_div(input_times_output_retained, u128::from(delta), slope_denom)?;
        let retained_denom = u128::from(NANOS_DENOM)
            .checked_mul(u128::from(NANOS_DENOM))
            .ok_or(ReserveV2ProgramErr::MathOverflow)?;
        let denom = retained_denom
            .checked_add(slope_term)
            .ok_or(ReserveV2ProgramErr::MathOverflow)?;
        let numerator = input_times_output_retained
            .checked_mul(entry_retained_nanos)
            .ok_or(ReserveV2ProgramErr::MathOverflow)?;
        let output = numerator / denom;
        let output: u64 = output
            .try_into()
            .map_err(|_| ReserveV2ProgramErr::MathOverflow)?;
        if output > max_piece_output {
            return Err(ReserveV2ProgramErr::MathOverflow);
        }
        Ok(output)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Band {
    start_used: u64,
    end_used: u64,
    start_fee_nanos: FeeNanos,
    end_fee_nanos: FeeNanos,
}

impl Band {
    #[inline]
    fn width(&self) -> Result<u64, ReserveV2ProgramErr> {
        match self.end_used.checked_sub(self.start_used) {
            Some(width) if width > 0 => Ok(width),
            _ => Err(ReserveV2ProgramErr::MathOverflow),
        }
    }

    #[inline]
    fn delta(&self) -> Result<u32, ReserveV2ProgramErr> {
        self.end_fee_nanos
            .get()
            .checked_sub(self.start_fee_nanos.get())
            .ok_or(ReserveV2ProgramErr::MathOverflow)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum ExactInPiece {
    Full { output: u64, input_used: u64 },
    Partial { output: u64 },
}

impl PriceExactIn for RangeOutPricing {
    type Error = ReserveV2ProgramErr;

    #[inline]
    fn price_exact_in(
        &self,
        PriceExactInIxArgs { sol_value, .. }: PriceExactInIxArgs,
    ) -> Result<u64, Self::Error> {
        self.pp_price_exact_in(sol_value)
    }
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

#[allow(deprecated)]
impl PriceLpTokensToMint for RangeOutPricing {
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
impl PriceLpTokensToRedeem for RangeOutPricing {
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

    fn price_exact_in(
        pricing: &RangeOutPricing,
        sol_value: u64,
    ) -> Result<u64, ReserveV2ProgramErr> {
        pricing.price_exact_in(PriceExactInIxArgs { amt: 0, sol_value })
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
    fn over_liquid_state_is_rejected() {
        let pool_sol_value = TEST_POOL_SOL_VALUE;
        let wsol_balance = 1_050_000 * LAMPORTS_PER_SOL;
        let pricing = range_out_pricing(pool_sol_value, wsol_balance);

        assert_eq!(
            price_exact_out(&pricing, 10_000 * LAMPORTS_PER_SOL),
            Err(ReserveV2ProgramErr::WsolBalanceGtPoolSolValue(
                WsolBalanceGtPoolSolValueErr {
                    pool_sol_value,
                    wsol_balance,
                },
            ))
        );
        assert_eq!(
            price_exact_in(&pricing, 10_000 * LAMPORTS_PER_SOL),
            Err(ReserveV2ProgramErr::WsolBalanceGtPoolSolValue(
                WsolBalanceGtPoolSolValueErr {
                    pool_sol_value,
                    wsol_balance,
                },
            ))
        );
    }

    #[test]
    fn exact_in_band_1() {
        let pricing = range_out_pricing(TEST_POOL_SOL_VALUE, TEST_POOL_SOL_VALUE);

        assert_eq!(
            price_exact_in(&pricing, 100_000 * LAMPORTS_PER_SOL),
            Ok(99_750_623_341_645)
        );
    }

    #[test]
    fn exact_in_crosses_threshold() {
        let pricing = range_out_pricing(TEST_POOL_SOL_VALUE, 850_000 * LAMPORTS_PER_SOL);

        assert_eq!(
            price_exact_in(&pricing, 100_000 * LAMPORTS_PER_SOL),
            Ok(98_926_660_104_296)
        );
    }

    #[test]
    fn exact_in_band_2() {
        let pricing = range_out_pricing(TEST_POOL_SOL_VALUE, 450_000 * LAMPORTS_PER_SOL);

        assert_eq!(
            price_exact_in(&pricing, 100_000 * LAMPORTS_PER_SOL),
            Ok(94_530_764_350_528)
        );
    }

    #[test]
    fn exact_in_full_drain_boundary() {
        let pricing = range_out_pricing(TEST_POOL_SOL_VALUE, TEST_POOL_SOL_VALUE);
        let full_drain_cost = price_exact_out(&pricing, TEST_POOL_SOL_VALUE).unwrap();

        assert_eq!(
            price_exact_in(&pricing, full_drain_cost),
            Ok(TEST_POOL_SOL_VALUE)
        );
        assert_eq!(
            price_exact_in(&pricing, full_drain_cost + 1),
            Err(ReserveV2ProgramErr::OverCap(OverCapErr {
                requested_out_sol_value: TEST_POOL_SOL_VALUE + 1,
                wsol_balance: TEST_POOL_SOL_VALUE,
            }))
        );
    }

    #[test]
    fn exact_in_zero_retained_value() {
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
            price_exact_in(&pricing, 1),
            Err(ReserveV2ProgramErr::ZeroRetainedValue)
        );
    }

    #[test]
    fn exact_in_zero_input() {
        let pricing = range_out_pricing(TEST_POOL_SOL_VALUE, TEST_POOL_SOL_VALUE);

        assert_eq!(price_exact_in(&pricing, 0), Ok(0));
    }

    #[test]
    fn exact_in_buffer_created_zero_retained_returns_zero() {
        let pricing = RangeOutPricing {
            input_fee_curve: InputFeeCurve {
                base_fee_nanos: FeeNanos::new(NANOS_DENOM - 1).unwrap(),
                threshold_nanos: ThresholdNanos::new(TEST_THRESHOLD_NANOS).unwrap(),
                threshold_fee_nanos: FeeNanos::MAX,
                max_fee_nanos: FeeNanos::MAX,
            },
            output_fee_nanos: FeeNanos::ZERO,
            pool_sol_value: TEST_POOL_SOL_VALUE,
            wsol_balance: TEST_POOL_SOL_VALUE,
        };

        assert_eq!(price_exact_in(&pricing, u64::from(NANOS_DENOM)), Ok(0));
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
                let wsol_balance = wsol_balance.min(pool_sol_value);
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

        #[test]
        fn zero_fee_exact_in_eq_input(
            pool_sol_value in 1..=u64::MAX,
            wsol_balance: u64,
            input_sol_value: u64,
        ) {
            let wsol_balance = wsol_balance.min(pool_sol_value);
            let pricing = RangeOutPricing {
                input_fee_curve: InputFeeCurve {
                    base_fee_nanos: FeeNanos::ZERO,
                    threshold_nanos: ThresholdNanos::new(TEST_THRESHOLD_NANOS).unwrap(),
                    threshold_fee_nanos: FeeNanos::ZERO,
                    max_fee_nanos: FeeNanos::ZERO,
                },
                output_fee_nanos: FeeNanos::ZERO,
                pool_sol_value,
                wsol_balance,
            };
            let expected = if input_sol_value <= wsol_balance {
                Ok(input_sol_value)
            } else {
                Err(ReserveV2ProgramErr::OverCap(OverCapErr {
                    requested_out_sol_value: wsol_balance.saturating_add(1),
                    wsol_balance,
                }))
            };

            prop_assert_eq!(price_exact_in(&pricing, input_sol_value), expected);
        }

        #[test]
        fn exact_in_success_is_safe(
            pricing in range_out_pricing_props(),
            input_sol_value: u64,
        ) {
            let output_sol_value = match price_exact_in(&pricing, input_sol_value) {
                Ok(output_sol_value) => output_sol_value,
                Err(_) => return Ok(()),
            };
            let required_input = price_exact_out(&pricing, output_sol_value)
                .map_err(|e| TestCaseError::fail(format!("exact-out failed for exact-in output: {e}")))?;

            prop_assert!(required_input <= input_sol_value);
            prop_assert!(output_sol_value <= pricing.wsol_balance);
        }

        #[test]
        fn exact_in_forward_check_never_fires(
            pricing in range_out_pricing_props(),
            input_sol_value: u64,
        ) {
            prop_assert!(!matches!(
                price_exact_in(&pricing, input_sol_value),
                Err(ReserveV2ProgramErr::ExactInForwardCheckFailed(_))
            ));
        }
    }
}
