use inf1_pp_core::{instructions::price::exact_in::PriceExactInIxArgs, traits::main::PriceExactIn};
use inf1_pp_flatfee_core::pricing::price::FlatFeeSwapPricing;

use crate::{pricing::common::PricingAgErr, PricingAg};

pub type PriceExactInAg = PricingAg<FlatFeeSwapPricing>;

pub type PriceExactInAgErr = PricingAgErr;

impl PriceExactIn for PriceExactInAg {
    type Error = PriceExactInAgErr;

    #[inline]
    fn price_exact_in(&self, args: PriceExactInIxArgs) -> Result<u64, Self::Error> {
        match self {
            Self::FlatFee(p) => p.price_exact_in(args).map_err(PriceExactInAgErr::FlatFee),
        }
    }
}
