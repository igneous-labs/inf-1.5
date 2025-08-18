#![allow(deprecated)]

use std::convert::Infallible;

use inf1_pp_flatfee_core::{
    instructions::pricing::lp::{mint::FlatFeeMintLpAccs, redeem::FlatFeeRedeemLpAccs},
    pricing::lp::{FlatFeeMintLpPricing, FlatFeeRedeemLpPricing},
};
use inf1_pp_std::traits::deprecated::{
    PriceLpTokensToMintAccsCol, PriceLpTokensToMintCol, PriceLpTokensToRedeemAccsCol,
    PriceLpTokensToRedeemCol,
};

use crate::{traits::FlatFeePricingColErr, FlatFeePricing};

// Quoting

impl<F, C> FlatFeePricing<F, C> {
    #[inline]
    pub const fn flat_fee_mint_lp_pricing(&self) -> FlatFeeMintLpPricing {
        FlatFeeMintLpPricing
    }
}

impl<F, C> PriceLpTokensToMintCol for FlatFeePricing<F, C> {
    type Error = Infallible;
    type PriceLpTokensToMint = FlatFeeMintLpPricing;

    #[inline]
    fn price_lp_tokens_to_mint_for(
        &self,
        _inp_mint: &[u8; 32],
    ) -> Result<Self::PriceLpTokensToMint, Self::Error> {
        Ok(self.flat_fee_mint_lp_pricing())
    }
}

impl<F, C> FlatFeePricing<F, C> {
    #[inline]
    pub const fn flat_fee_redeem_lp_pricing(&self) -> Option<FlatFeeRedeemLpPricing> {
        match self.lp_withdrawal_fee_bps {
            None => None,
            Some(lp_withdrawal_fee_bps) => Some(FlatFeeRedeemLpPricing {
                lp_withdrawal_fee_bps,
            }),
        }
    }
}

impl<F, C> PriceLpTokensToRedeemCol for FlatFeePricing<F, C> {
    type Error = FlatFeePricingColErr;
    type PriceLpTokensToRedeem = FlatFeeRedeemLpPricing;

    #[inline]
    fn price_lp_tokens_to_redeem_for(
        &self,
        _out_mint: &[u8; 32],
    ) -> Result<Self::PriceLpTokensToRedeem, Self::Error> {
        self.flat_fee_redeem_lp_pricing()
            .ok_or(FlatFeePricingColErr::ProgramStateMissing)
    }
}

// Accounts

impl<F, C> FlatFeePricing<F, C> {
    #[inline]
    pub const fn flat_fee_mint_lp_accs(&self) -> FlatFeeMintLpAccs {
        FlatFeeMintLpAccs
    }
}

impl<F, C> PriceLpTokensToMintAccsCol for FlatFeePricing<F, C> {
    type Error = Infallible;
    type PriceLpTokensToMintAccs = FlatFeeMintLpAccs;

    fn price_lp_tokens_to_mint_accs_for(
        &self,
        _inp_mint: &[u8; 32],
    ) -> Result<Self::PriceLpTokensToMintAccs, Self::Error> {
        Ok(self.flat_fee_mint_lp_accs())
    }
}

impl<F, C> FlatFeePricing<F, C> {
    #[inline]
    pub const fn flat_fee_redeem_lp_accs(&self) -> FlatFeeRedeemLpAccs {
        FlatFeeRedeemLpAccs::MAINNET
    }
}

impl<F, C> PriceLpTokensToRedeemAccsCol for FlatFeePricing<F, C> {
    type Error = Infallible;
    type PriceLpTokensToRedeemAccs = FlatFeeRedeemLpAccs;

    #[inline]
    fn price_lp_tokens_to_redeem_accs_for(
        &self,
        _out_mint: &[u8; 32],
    ) -> Result<Self::PriceLpTokensToRedeemAccs, Self::Error> {
        Ok(self.flat_fee_redeem_lp_accs())
    }
}
