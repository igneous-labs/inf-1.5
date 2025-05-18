use crate::instructions::{
    lp::{mint::PriceLpTokensToMintIxArgs, redeem::PriceLpTokensToRedeemIxArgs},
    price::{exact_in::PriceExactInIxArgs, exact_out::PriceExactOutIxArgs},
};

pub trait PriceExactIn {
    type Error;

    fn price_exact_in(&self, input: PriceExactInIxArgs) -> Result<u64, Self::Error>;
}

pub trait PriceExactOut {
    type Error;

    fn price_exact_out(&self, output: PriceExactOutIxArgs) -> Result<u64, Self::Error>;
}

pub trait PriceLpTokensToMint {
    type Error;

    fn price_lp_tokens_to_mint(&self, input: PriceLpTokensToMintIxArgs)
        -> Result<u64, Self::Error>;
}

pub trait PriceLpTokensToRedeem {
    type Error;

    fn price_lp_tokens_to_redeem(
        &self,
        input: PriceLpTokensToRedeemIxArgs,
    ) -> Result<u64, Self::Error>;
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
