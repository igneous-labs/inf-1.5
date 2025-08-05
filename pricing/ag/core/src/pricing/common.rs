use core::{error::Error, fmt::Display};

use inf1_pp_flatfee_core::pricing::err::FlatFeePricingErr;

use crate::PricingAg;

// TODO: this definition might diverge once other variants have more complex
// error types that resultin different generic args depending on the pricing trait used
pub type PricingAgErr = PricingAg<FlatFeePricingErr>;

impl Display for PricingAgErr {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::FlatFee(e) => e.fmt(f),
        }
    }
}

impl Error for PricingAgErr {}
