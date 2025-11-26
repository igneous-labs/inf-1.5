use inf1_ctl_core::typedefs::{
    fee_nanos::{FeeNanos, MAX_FEE_NANOS},
    rps::{Rps, MIN_RPS_RAW},
    uq0f63::UQ0F63,
};
use proptest::prelude::Strategy;

/// copy-pastad from core
pub fn any_ctl_fee_nanos_strat() -> impl Strategy<Value = FeeNanos> {
    (0..=MAX_FEE_NANOS)
        .prop_map(FeeNanos::new)
        .prop_map(Result::unwrap)
}

/// copy-pastad from core
pub fn any_rps_strat() -> impl Strategy<Value = Rps> {
    (MIN_RPS_RAW..=*UQ0F63::ONE.as_raw())
        .prop_map(UQ0F63::new)
        .prop_map(Result::unwrap)
        .prop_map(Rps::new)
        .prop_map(Result::unwrap)
}
