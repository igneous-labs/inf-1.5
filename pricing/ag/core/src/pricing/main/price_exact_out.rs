use inf1_pp_core::{
    instructions::price::exact_out::PriceExactOutIxArgs, traits::main::PriceExactOut,
};
use inf1_pp_flatfee_core::pricing::price::FlatFeeSwapPricing;

use crate::{pricing::err::PricingAgErr, PricingAg};

pub type PriceExactOutAg = PricingAg<FlatFeeSwapPricing>;

pub type PriceExactOutAgErr = PricingAgErr;

impl PriceExactOut for PriceExactOutAg {
    type Error = PriceExactOutAgErr;

    #[inline]
    fn price_exact_out(&self, args: PriceExactOutIxArgs) -> Result<u64, Self::Error> {
        match self {
            Self::FlatFee(p) => p.price_exact_out(args).map_err(PriceExactOutAgErr::FlatFee),
        }
    }
}
