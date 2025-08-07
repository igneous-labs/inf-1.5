use inf1_pp_flatfee_core::pricing::err::FlatFeePricingErr;

use crate::PricingAg;

// TODO: this definition might diverge once other variants have more complex
// error types that result in different generic args depending on the pricing trait used,
// resulting in the need for distinct types like PriceExactInAgErr, PriceExactOutAgErr instead
pub type PricingAgErr = PricingAg<FlatFeePricingErr>;
