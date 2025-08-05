use inf1_pp_core::{
    instructions::deprecated::lp::redeem::PriceLpTokensToRedeemIxArgs,
    traits::deprecated::PriceLpTokensToRedeem,
};
use inf1_pp_flatfee_core::pricing::lp::FlatFeeRedeemLpPricing;

use crate::{pricing::common::PricingAgErr, PricingAccsAg};

pub type PriceRedeemLpAg = PricingAccsAg<FlatFeeRedeemLpPricing>;

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
        }
    }
}
