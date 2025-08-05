use core::{error::Error, fmt::Display};

use inf1_pp_core::{
    instructions::price::exact_out::PriceExactOutIxArgs, traits::main::PriceExactOut,
};
use inf1_pp_flatfee_core::pricing::{err::FlatFeePricingErr, price::FlatFeeSwapPricing};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PriceExactOutAg {
    FlatFee(FlatFeeSwapPricing),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PriceExactOutAgErr {
    FlatFee(FlatFeePricingErr),
}

impl Display for PriceExactOutAgErr {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::FlatFee(e) => e.fmt(f),
        }
    }
}

impl Error for PriceExactOutAgErr {}

impl PriceExactOut for PriceExactOutAg {
    type Error = PriceExactOutAgErr;

    #[inline]
    fn price_exact_out(&self, args: PriceExactOutIxArgs) -> Result<u64, Self::Error> {
        match self {
            Self::FlatFee(p) => p.price_exact_out(args).map_err(PriceExactOutAgErr::FlatFee),
        }
    }
}
