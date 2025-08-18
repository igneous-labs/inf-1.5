use inf1_pp_core::{instructions::price::exact_in::PriceExactInIxArgs, traits::main::PriceExactIn};
use inf1_pp_flatfee_core::pricing::price::FlatFeeSwapPricing;
use inf1_pp_flatslab_core::pricing::FlatSlabSwapPricing;

use crate::{internal_utils::map_variant_err, pricing::err::PricingAgErr, PricingAg};

pub type PriceExactInAg = PricingAg<FlatFeeSwapPricing, FlatSlabSwapPricing>;

pub type PriceExactInAgErr = PricingAgErr;

impl PriceExactIn for PriceExactInAg {
    type Error = PriceExactInAgErr;

    #[inline]
    fn price_exact_in(&self, args: PriceExactInIxArgs) -> Result<u64, Self::Error> {
        map_variant_err!(self, (|p| PriceExactIn::price_exact_in(p, args)))
    }
}
