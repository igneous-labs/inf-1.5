use inf1_pp_flatfee_core::pricing::err::FlatFeePricingErr;
use inf1_pp_flatslab_core::pricing::FlatSlabPricingErr;

use crate::PricingAg;

pub type PricingAgErr = PricingAg<FlatFeePricingErr, FlatSlabPricingErr>;
