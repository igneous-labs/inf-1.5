//! Main pricing program traits.
//!
//! These traits are [`crate::pair::Pair`]-agnostic, and are focused around implementing the interface
//! for a single, specific `Pair`.
//!
//! For traits that are parameterized across `Pair`s, which is more representative of an entire pricing program,
//! see [`super::collection`]

use core::ops::Deref;

use crate::instructions::price::{exact_in::PriceExactInIxArgs, exact_out::PriceExactOutIxArgs};

// Quoting

pub trait PriceExactIn {
    type Error: core::error::Error;

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
    type Error: core::error::Error;

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

// Accounts

/// Suffix account meta slices returned by the 3 methods
/// - must all have the same length
/// - must all have length <= u8::MAX
///
/// Append the suffix to the prefixes [`crate::instructions::price::exact_in::PriceExactInIxPreAccs`] to create
/// the account inputs of a full interface instruction
pub trait PriceExactInAccs {
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

/// Suffix account meta slices returned by the 3 methods
/// - must all have the same length
/// - must all have length <= u8::MAX
///
/// Append the suffix to the prefixes [`crate::instructions::price::exact_out::PriceExactOutIxPreAccs`] to create
/// the account inputs of a full interface instruction
pub trait PriceExactOutAccs {
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
