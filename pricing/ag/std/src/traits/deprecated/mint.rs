use inf1_pp_ag_core::{instructions::PriceLpTokensToMintAccsAg, pricing::PriceMintLpAg, PricingAg};
use inf1_pp_std::traits::deprecated::{PriceLpTokensToMintAccsCol, PriceLpTokensToMintCol};

use crate::{PricingProgAg, PricingProgAgErr};

impl<F, C> PriceLpTokensToMintCol for PricingProgAg<F, C> {
    type Error = PricingProgAgErr;
    type PriceLpTokensToMint = PriceMintLpAg;

    #[inline]
    fn price_lp_tokens_to_mint_for(
        &self,
        inp_mint: &[u8; 32],
    ) -> Result<Self::PriceLpTokensToMint, Self::Error> {
        match &self.0 {
            PricingAg::FlatFee(p) => Ok(p
                .price_lp_tokens_to_mint_for(inp_mint)
                .map(PricingAg::FlatFee)
                .unwrap()), // unwrap-safety: infallible
        }
    }
}

impl<F, C> PriceLpTokensToMintAccsCol for PricingProgAg<F, C> {
    type Error = PricingProgAgErr;
    type PriceLpTokensToMintAccs = PriceLpTokensToMintAccsAg;

    fn price_lp_tokens_to_mint_accs_for(
        &self,
        inp_mint: &[u8; 32],
    ) -> Result<Self::PriceLpTokensToMintAccs, Self::Error> {
        match &self.0 {
            PricingAg::FlatFee(p) => Ok(p
                .price_lp_tokens_to_mint_accs_for(inp_mint)
                .map(PricingAg::FlatFee)
                .unwrap()), // unwrap-safety: infallible
        }
    }
}
