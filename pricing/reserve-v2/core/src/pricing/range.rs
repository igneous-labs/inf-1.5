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
