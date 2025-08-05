use core::{error::Error, fmt::Display};

use inf1_pp_core::{
    instructions::deprecated::lp::redeem::PriceLpTokensToRedeemIxArgs,
    traits::deprecated::PriceLpTokensToRedeem,
};
use inf1_pp_flatfee_core::pricing::{err::FlatFeePricingErr, lp::FlatFeeRedeemLpPricing};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PriceRedeemLpAg {
    FlatFee(FlatFeeRedeemLpPricing),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PriceRedeemLpAgErr {
    FlatFee(FlatFeePricingErr),
}

impl Display for PriceRedeemLpAgErr {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::FlatFee(e) => e.fmt(f),
        }
    }
}

impl Error for PriceRedeemLpAgErr {}

impl PriceLpTokensToRedeem for PriceRedeemLpAg {
    type Error = PriceRedeemLpAgErr;

    fn price_lp_tokens_to_redeem(
        &self,
        input: PriceLpTokensToRedeemIxArgs,
    ) -> Result<u64, Self::Error> {
        match self {
            Self::FlatFee(p) => p
                .price_lp_tokens_to_redeem(input)
                .map_err(PriceRedeemLpAgErr::FlatFee),
        }
    }
}
