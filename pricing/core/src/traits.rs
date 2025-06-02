use core::ops::Deref;

use crate::instructions::{
    lp::{mint::PriceLpTokensToMintIxArgs, redeem::PriceLpTokensToRedeemIxArgs},
    price::{exact_in::PriceExactInIxArgs, exact_out::PriceExactOutIxArgs},
};

pub trait PriceExactIn {
    type Error;

    fn price_exact_in(&self, input: PriceExactInIxArgs) -> Result<u64, Self::Error>;
}

/// Blanket for refs
impl<R, T: PriceExactIn> PriceExactIn for R
where
    R: Deref<Target = T>,
{
    type Error = T::Error;

    #[inline]
    fn price_exact_in(&self, input: PriceExactInIxArgs) -> Result<u64, Self::Error> {
        self.deref().price_exact_in(input)
    }
}

pub trait PriceExactOut {
    type Error;

    fn price_exact_out(&self, output: PriceExactOutIxArgs) -> Result<u64, Self::Error>;
}

/// Blanket for refs
impl<R, T: PriceExactOut> PriceExactOut for R
where
    R: Deref<Target = T>,
{
    type Error = T::Error;

    #[inline]
    fn price_exact_out(&self, output: PriceExactOutIxArgs) -> Result<u64, Self::Error> {
        self.deref().price_exact_out(output)
    }
}

pub trait PriceLpTokensToMint {
    type Error;

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
    type Error;

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

/// Suffix account meta slices returned by the 3 methods must all have the same length.
///
/// Append the suffix to the prefixes [`crate::instructions::price::exact_in::PriceExactInIxPreAccs`] to create
/// the account inputs of a full interface instruction
pub trait PriceExactInAccs {
    type KeysOwned: AsRef<[[u8; 32]]>;
    type AccFlags: AsRef<[bool]>;

    fn suf_keys_owned(&self) -> Self::KeysOwned;
    fn suf_is_writer(&self) -> Self::AccFlags;
    fn suf_is_signer(&self) -> Self::AccFlags;
}

/// Blanket for refs
impl<R, T: PriceExactInAccs> PriceExactInAccs for R
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

/// Suffix account meta slices returned by the 3 methods must all have the same length.
///
/// Append the suffix to the prefixes [`crate::instructions::price::exact_out::PriceExactOutIxPreAccs`] to create
/// the account inputs of a full interface instruction
pub trait PriceExactOutAccs {
    type KeysOwned: AsRef<[[u8; 32]]>;
    type AccFlags: AsRef<[bool]>;

    fn suf_keys_owned(&self) -> Self::KeysOwned;
    fn suf_is_writer(&self) -> Self::AccFlags;
    fn suf_is_signer(&self) -> Self::AccFlags;
}

/// Blanket for refs
impl<R, T: PriceExactOutAccs> PriceExactOutAccs for R
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

/// Suffix account meta slices returned by the 3 methods must all have the same length.
///
/// Append the suffix to the prefixes [`crate::instructions::lp::mint::PriceLpTokensToMintIxPreAccs`] to create
/// the account inputs of a full interface instruction
pub trait PriceLpTokensToMintAccs {
    type KeysOwned: AsRef<[[u8; 32]]>;
    type AccFlags: AsRef<[bool]>;

    fn suf_keys_owned(&self) -> Self::KeysOwned;
    fn suf_is_writer(&self) -> Self::AccFlags;
    fn suf_is_signer(&self) -> Self::AccFlags;
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

/// Suffix account meta slices returned by the 3 methods must all have the same length.
///
/// Append the suffix to the prefixes [`crate::instructions::lp::redeem::PriceLpTokensToRedeemIxPreAccs`] to create
/// the account inputs of a full interface instruction
pub trait PriceLpTokensToRedeemAccs {
    type KeysOwned: AsRef<[[u8; 32]]>;
    type AccFlags: AsRef<[bool]>;

    fn suf_keys_owned(&self) -> Self::KeysOwned;
    fn suf_is_writer(&self) -> Self::AccFlags;
    fn suf_is_signer(&self) -> Self::AccFlags;
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
