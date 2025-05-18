use core::convert::Infallible;

use inf1_pricing_core::{
    instructions::lp::{mint::PriceLpTokensToMintIxArgs, redeem::PriceLpTokensToRedeemIxArgs},
    traits::{PriceLpTokensToMint, PriceLpTokensToRedeem},
};
use sanctum_fee_ratio::{AftFee, Fee};
use sanctum_u64_ratio::{Ceil, Ratio};

use super::err::FlatFeePricingErr;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FlatFeeMintLpPricing;

impl FlatFeeMintLpPricing {
    /// no-op, returns the same sol_value since no fees are charged on minting LP tokens
    #[inline]
    pub const fn pp_price_lp_tokens_to_mint(sol_value: u64) -> u64 {
        sol_value
    }
}

impl PriceLpTokensToMint for FlatFeeMintLpPricing {
    type Error = Infallible;

    #[inline]
    fn price_lp_tokens_to_mint(
        &self,
        PriceLpTokensToMintIxArgs { sol_value, .. }: PriceLpTokensToMintIxArgs,
    ) -> Result<u64, Self::Error> {
        Ok(Self::pp_price_lp_tokens_to_mint(sol_value))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct FlatFeeRedeemLpPricing {
    /// Read from [`crate::accounts::program_state::ProgramState`]
    pub lp_withdrawal_fee_bps: u16,
}

type Fcr = Fee<Ceil<Ratio<u16, u16>>>;

impl FlatFeeRedeemLpPricing {
    /// Returns None if self's data results in an invalid fee
    #[inline]
    pub const fn fee(&self) -> Option<Fcr> {
        Fcr::new(Ratio {
            n: self.lp_withdrawal_fee_bps,
            d: 10_000,
        })
    }

    #[inline]
    pub const fn pp_price_lp_tokens_to_redeem(&self, sol_value: u64) -> Option<AftFee> {
        match self.fee() {
            None => None,
            Some(f) => f.apply(sol_value),
        }
    }
}

impl PriceLpTokensToRedeem for FlatFeeRedeemLpPricing {
    type Error = FlatFeePricingErr;

    #[inline]
    fn price_lp_tokens_to_redeem(
        &self,
        PriceLpTokensToRedeemIxArgs { sol_value, .. }: PriceLpTokensToRedeemIxArgs,
    ) -> Result<u64, Self::Error> {
        self.pp_price_lp_tokens_to_redeem(sol_value)
            .map(|aaf| aaf.rem())
            .ok_or(FlatFeePricingErr::Ratio)
    }
}
