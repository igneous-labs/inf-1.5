use inf1_pp_ag_core::{
    instructions::PriceLpTokensToRedeemAccsAg, pricing::PriceRedeemLpAg, PricingAg,
};
use inf1_pp_std::traits::deprecated::{PriceLpTokensToRedeemAccsCol, PriceLpTokensToRedeemCol};

use crate::{PricingProgAg, PricingProgAgErr, PricingProgAgInfallibleErr};

impl<F, C> PriceLpTokensToRedeemCol for PricingProgAg<F, C> {
    type Error = PricingProgAgErr;
    type PriceLpTokensToRedeem = PriceRedeemLpAg;

    #[inline]
    fn price_lp_tokens_to_redeem_for(
        &self,
        out_mint: &[u8; 32],
    ) -> Result<Self::PriceLpTokensToRedeem, Self::Error> {
        match &self.0 {
            PricingAg::FlatFee(p) => p
                .price_lp_tokens_to_redeem_for(out_mint)
                .map(PricingAg::FlatFee)
                .map_err(PricingAg::FlatFee),
        }
    }
}

impl<F, C> PriceLpTokensToRedeemAccsCol for PricingProgAg<F, C> {
    type Error = PricingProgAgInfallibleErr;
    type PriceLpTokensToRedeemAccs = PriceLpTokensToRedeemAccsAg;

    fn price_lp_tokens_to_redeem_accs_for(
        &self,
        out_mint: &[u8; 32],
    ) -> Result<Self::PriceLpTokensToRedeemAccs, Self::Error> {
        match &self.0 {
            PricingAg::FlatFee(p) => p
                .price_lp_tokens_to_redeem_accs_for(out_mint)
                .map(PricingAg::FlatFee)
                .map_err(PricingAg::FlatFee),
        }
    }
}
