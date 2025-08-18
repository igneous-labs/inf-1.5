use inf1_pp_core::{
    instructions::price::exact_out::PriceExactOutIxArgs, traits::main::PriceExactOut,
};
use inf1_pp_flatfee_core::pricing::price::FlatFeeSwapPricing;
use inf1_pp_flatslab_core::pricing::FlatSlabSwapPricing;

use crate::{internal_utils::map_variant_err, pricing::err::PricingAgErr, PricingAg};

pub type PriceExactOutAg = PricingAg<FlatFeeSwapPricing, FlatSlabSwapPricing>;

pub type PriceExactOutAgErr = PricingAgErr;

impl PriceExactOut for PriceExactOutAg {
    type Error = PriceExactOutAgErr;

    #[inline]
    fn price_exact_out(&self, args: PriceExactOutIxArgs) -> Result<u64, Self::Error> {
        map_variant_err!(self, (|p| PriceExactOut::price_exact_out(p, args)))
    }
}
