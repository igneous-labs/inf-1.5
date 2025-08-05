use core::{error::Error, fmt::Display};

use inf1_pp_core::{instructions::price::exact_in::PriceExactInIxArgs, traits::main::PriceExactIn};
use inf1_pp_flatfee_core::pricing::{err::FlatFeePricingErr, price::FlatFeeSwapPricing};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PriceExactInAg {
    FlatFee(FlatFeeSwapPricing),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PriceExactInAgErr {
    FlatFee(FlatFeePricingErr),
}

impl Display for PriceExactInAgErr {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::FlatFee(e) => e.fmt(f),
        }
    }
}

impl Error for PriceExactInAgErr {}

impl PriceExactIn for PriceExactInAg {
    type Error = PriceExactInAgErr;

    #[inline]
    fn price_exact_in(&self, args: PriceExactInIxArgs) -> Result<u64, Self::Error> {
        match self {
            Self::FlatFee(p) => p.price_exact_in(args).map_err(PriceExactInAgErr::FlatFee),
        }
    }
}
