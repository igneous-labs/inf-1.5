use inf1_pp_core::{
    instructions::deprecated::lp::redeem::PriceLpTokensToRedeemIxArgs,
    traits::deprecated::PriceLpTokensToRedeem,
};
use inf1_pp_flatfee_core::pricing::lp::FlatFeeRedeemLpPricing;
use inf1_pp_flatslab_core::pricing::FlatSlabSwapPricing;

use crate::{internal_utils::map_variant_err, pricing::err::PricingAgErr, PricingAg};

pub type PriceRedeemLpAg = PricingAg<FlatFeeRedeemLpPricing, FlatSlabSwapPricing>;

pub type PriceRedeemLpAgErr = PricingAgErr;

impl PriceLpTokensToRedeem for PriceRedeemLpAg {
    type Error = PriceRedeemLpAgErr;

    fn price_lp_tokens_to_redeem(
        &self,
        input: PriceLpTokensToRedeemIxArgs,
    ) -> Result<u64, Self::Error> {
        map_variant_err!(
            self,
            (|p| PriceLpTokensToRedeem::price_lp_tokens_to_redeem(p, input))
        )
    }
}
