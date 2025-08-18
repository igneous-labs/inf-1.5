use inf1_pp_core::{
    instructions::deprecated::lp::redeem::PriceLpTokensToRedeemIxArgs,
    traits::deprecated::PriceLpTokensToRedeem,
};
use inf1_pp_flatfee_core::pricing::lp::FlatFeeRedeemLpPricing;
use inf1_pp_flatslab_core::pricing::FlatSlabSwapPricing;

use crate::{pricing::err::PricingAgErr, PricingAg};

pub type PriceRedeemLpAg = PricingAg<FlatFeeRedeemLpPricing, FlatSlabSwapPricing>;

pub type PriceRedeemLpAgErr = PricingAgErr;

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
            Self::FlatSlab(p) => p
                .price_lp_tokens_to_redeem(input)
                .map_err(PriceRedeemLpAgErr::FlatSlab),
        }
    }
}
