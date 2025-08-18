use inf1_pp_core::{
    instructions::deprecated::lp::mint::PriceLpTokensToMintIxArgs,
    traits::deprecated::PriceLpTokensToMint,
};
use inf1_pp_flatfee_core::pricing::lp::FlatFeeMintLpPricing;
use inf1_pp_flatslab_core::pricing::FlatSlabSwapPricing;

use crate::{internal_utils::map_variant_err, pricing::err::PricingAgErr, PricingAg};

pub type PriceMintLpAg = PricingAg<FlatFeeMintLpPricing, FlatSlabSwapPricing>;

pub type PriceMintLpAgErr = PricingAgErr;

impl PriceLpTokensToMint for PriceMintLpAg {
    type Error = PriceMintLpAgErr;

    fn price_lp_tokens_to_mint(
        &self,
        input: PriceLpTokensToMintIxArgs,
    ) -> Result<u64, Self::Error> {
        map_variant_err!(
            self,
            (|p| PriceLpTokensToMint::price_lp_tokens_to_mint(p, input))
        )
    }
}
