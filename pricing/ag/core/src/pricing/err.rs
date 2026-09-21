use inf1_pp_flatfee_core::pricing::err::FlatFeePricingErr;
use inf1_pp_flatslab_core::pricing::FlatSlabPricingErr;
use inf1_pp_reserve_v2_core::{errs::ReserveV2ProgramErr, pricing::ReserveV2Swap};

use crate::PricingAg;

pub type PricingAgErr = PricingAg<
    FlatFeePricingErr,
    FlatSlabPricingErr,
    ReserveV2Swap<ReserveV2ProgramErr, ReserveV2ProgramErr>,
>;
