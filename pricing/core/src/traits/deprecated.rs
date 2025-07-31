#![allow(deprecated)]
#![deprecated(
    since = "0.2.0",
    note = r#"
The new pricing program interface has been simplified to only have PriceExactIn and PriceExactOut.
The LP token (INF) should simply be treated as any other token (output=INF <-> addLiquidity, input=INF <-> removeLiquidity). 
"#
)]

use core::ops::Deref;

use crate::instructions::deprecated::lp::{
    mint::PriceLpTokensToMintIxArgs, redeem::PriceLpTokensToRedeemIxArgs,
};

pub trait PriceLpTokensToMint {
    type Error: core::error::Error;

    fn price_lp_tokens_to_mint(&self, input: PriceLpTokensToMintIxArgs)
        -> Result<u64, Self::Error>;
}

/// Blanket for refs
impl<R, T: PriceLpTokensToMint> PriceLpTokensToMint for R
where
    R: Deref<Target = T>,
{
    type Error = T::Error;

    #[inline]
    fn price_lp_tokens_to_mint(
        &self,
        input: PriceLpTokensToMintIxArgs,
    ) -> Result<u64, Self::Error> {
        self.deref().price_lp_tokens_to_mint(input)
    }
}

pub trait PriceLpTokensToRedeem {
    type Error: core::error::Error;

    fn price_lp_tokens_to_redeem(
        &self,
        input: PriceLpTokensToRedeemIxArgs,
    ) -> Result<u64, Self::Error>;
}

/// Blanket for refs
impl<R, T: PriceLpTokensToRedeem> PriceLpTokensToRedeem for R
where
    R: Deref<Target = T>,
{
    type Error = T::Error;

    #[inline]
    fn price_lp_tokens_to_redeem(
        &self,
        input: PriceLpTokensToRedeemIxArgs,
    ) -> Result<u64, Self::Error> {
        self.deref().price_lp_tokens_to_redeem(input)
    }
}

/// Suffix account meta slices returned by the 3 methods
/// - must all have the same length
/// - must all have length <= u8::MAX
///
/// Append the suffix to the prefixes [`crate::instructions::lp::mint::PriceLpTokensToMintIxPreAccs`] to create
/// the account inputs of a full interface instruction
pub trait PriceLpTokensToMintAccs {
    type KeysOwned: AsRef<[[u8; 32]]>;
    type AccFlags: AsRef<[bool]>;

    fn suf_keys_owned(&self) -> Self::KeysOwned;
    fn suf_is_writer(&self) -> Self::AccFlags;
    fn suf_is_signer(&self) -> Self::AccFlags;

    #[inline]
    fn suf_len(&self) -> u8 {
        // unwrap-safety: there should not be a pricing program that uses more than 255 accounts
        self.suf_is_signer().as_ref().len().try_into().unwrap()
    }
}

/// Blanket for refs
impl<R, T: PriceLpTokensToMintAccs> PriceLpTokensToMintAccs for R
where
    R: Deref<Target = T>,
{
    type KeysOwned = T::KeysOwned;

    type AccFlags = T::AccFlags;

    #[inline]
    fn suf_keys_owned(&self) -> Self::KeysOwned {
        self.deref().suf_keys_owned()
    }

    #[inline]
    fn suf_is_writer(&self) -> Self::AccFlags {
        self.deref().suf_is_writer()
    }

    #[inline]
    fn suf_is_signer(&self) -> Self::AccFlags {
        self.deref().suf_is_signer()
    }
}

/// Suffix account meta slices returned by the 3 methods
/// - must all have the same length
/// - must all have length <= u8::MAX
///
/// Append the suffix to the prefixes [`crate::instructions::lp::redeem::PriceLpTokensToRedeemIxPreAccs`] to create
/// the account inputs of a full interface instruction
pub trait PriceLpTokensToRedeemAccs {
    type KeysOwned: AsRef<[[u8; 32]]>;
    type AccFlags: AsRef<[bool]>;

    fn suf_keys_owned(&self) -> Self::KeysOwned;
    fn suf_is_writer(&self) -> Self::AccFlags;
    fn suf_is_signer(&self) -> Self::AccFlags;

    #[inline]
    fn suf_len(&self) -> u8 {
        // unwrap-safety: there should not be a pricing program that uses more than 255 accounts
        self.suf_is_signer().as_ref().len().try_into().unwrap()
    }
}

/// Blanket for refs
impl<R, T: PriceLpTokensToRedeemAccs> PriceLpTokensToRedeemAccs for R
where
    R: Deref<Target = T>,
{
    type KeysOwned = T::KeysOwned;

    type AccFlags = T::AccFlags;

    #[inline]
    fn suf_keys_owned(&self) -> Self::KeysOwned {
        self.deref().suf_keys_owned()
    }

    #[inline]
    fn suf_is_writer(&self) -> Self::AccFlags {
        self.deref().suf_is_writer()
    }

    #[inline]
    fn suf_is_signer(&self) -> Self::AccFlags {
        self.deref().suf_is_signer()
    }
}
