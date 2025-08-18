#![allow(deprecated)]

use std::convert::Infallible;

use inf1_pp_flatslab_core::{
    instructions::pricing::FlatSlabPpAccs, keys::LP_MINT_ID, pricing::FlatSlabSwapPricing,
};
use inf1_pp_std::{
    pair::Pair,
    traits::deprecated::{
        PriceLpTokensToMintAccsCol, PriceLpTokensToMintCol, PriceLpTokensToRedeemAccsCol,
        PriceLpTokensToRedeemCol,
    },
};

use crate::{traits::FlatSlabPricingColErr, FlatSlabPricing};

// Quoting

impl PriceLpTokensToMintCol for FlatSlabPricing {
    type Error = FlatSlabPricingColErr;
    type PriceLpTokensToMint = FlatSlabSwapPricing;

    #[inline]
    fn price_lp_tokens_to_mint_for(
        &self,
        inp_mint: &[u8; 32],
    ) -> Result<Self::PriceLpTokensToMint, Self::Error> {
        self.flat_slab_swap_pricing_for(&Pair {
            inp: inp_mint,
            out: &LP_MINT_ID,
        })
        .map_err(Into::into)
    }
}

impl PriceLpTokensToRedeemCol for FlatSlabPricing {
    type Error = FlatSlabPricingColErr;
    type PriceLpTokensToRedeem = FlatSlabSwapPricing;

    #[inline]
    fn price_lp_tokens_to_redeem_for(
        &self,
        out_mint: &[u8; 32],
    ) -> Result<Self::PriceLpTokensToRedeem, Self::Error> {
        self.flat_slab_swap_pricing_for(&Pair {
            inp: &LP_MINT_ID,
            out: out_mint,
        })
        .map_err(Into::into)
    }
}

// Accounts

impl PriceLpTokensToMintAccsCol for FlatSlabPricing {
    type Error = Infallible;
    type PriceLpTokensToMintAccs = FlatSlabPpAccs;

    fn price_lp_tokens_to_mint_accs_for(
        &self,
        _inp_mint: &[u8; 32],
    ) -> Result<Self::PriceLpTokensToMintAccs, Self::Error> {
        Ok(self.flat_slab_pp_accs())
    }
}

impl PriceLpTokensToRedeemAccsCol for FlatSlabPricing {
    type Error = Infallible;
    type PriceLpTokensToRedeemAccs = FlatSlabPpAccs;

    fn price_lp_tokens_to_redeem_accs_for(
        &self,
        _out_mint: &[u8; 32],
    ) -> Result<Self::PriceLpTokensToRedeemAccs, Self::Error> {
        Ok(self.flat_slab_pp_accs())
    }
}
