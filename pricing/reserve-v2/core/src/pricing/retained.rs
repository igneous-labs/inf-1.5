use sanctum_u64_ratio::{Floor, Ratio};

use crate::{errs::ReserveV2ProgramErr, typedefs::FeeNanos};

#[inline]
pub(crate) fn price_exact_in_retained_product(
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
pub(crate) fn price_exact_out_retained_product(
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
pub(crate) fn route_retained_ratio(
    input_fee_nanos: FeeNanos,
    output_fee_nanos: FeeNanos,
) -> Result<Floor<Ratio<u64, u64>>, ReserveV2ProgramErr> {
    let input_retained_ratio = input_fee_nanos.retained_ratio();
    let output_retained_ratio = output_fee_nanos.retained_ratio();
    // retained numerators/denominators are <= NANOS_DENOM, so products fit in u64
    let retained_product = u64::from(input_retained_ratio.n) * u64::from(output_retained_ratio.n);
    if retained_product == 0 {
        return Err(ReserveV2ProgramErr::ZeroRetainedValue);
    }
    let denom = u64::from(input_retained_ratio.d) * u64::from(output_retained_ratio.d);
    Ok(Floor(Ratio {
        n: retained_product,
        d: denom,
    }))
}
