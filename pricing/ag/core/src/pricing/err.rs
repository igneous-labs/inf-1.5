use inf1_pp_flatfee_core::pricing::err::FlatFeePricingErr;

use crate::PricingAg;

pub type PricingAgErr = PricingAg<FlatFeePricingErr>;
