use generic_array_struct::generic_array_struct;
use inf1_pricing_core::traits::PriceLpTokensToMintProgram;

use crate::instructions::internal_utils::impl_asref;

/// This program has no additional accounts suffix
#[generic_array_struct(builder pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct PriceLpTokensToMintIxSufAccs<T> {}

impl<T> PriceLpTokensToMintIxSufAccs<T> {
    #[inline]
    pub const fn new() -> Self {
        Self([])
    }
}

pub type PriceLpTokensToMintIxSufKeys<'a> = PriceLpTokensToMintIxSufAccs<&'a [u8; 32]>;

pub type PriceLpTokensToMintIxSufKeysOwned = PriceLpTokensToMintIxSufAccs<[u8; 32]>;

pub type PriceLpTokensToMintIxSufAccFlags = PriceLpTokensToMintIxSufAccs<bool>;

impl_asref!(PriceLpTokensToMintIxSufAccs<T>);

impl PriceLpTokensToMintProgram for PriceLpTokensToMintIxSufKeysOwned {
    type KeysOwned = Self;
    type AccFlags = PriceLpTokensToMintIxSufAccFlags;

    #[inline]
    fn suf_keys_owned(&self) -> Self::KeysOwned {
        *self
    }

    #[inline]
    fn suf_is_writer(&self) -> Self::AccFlags {
        Self::AccFlags::new()
    }

    #[inline]
    fn suf_is_signer(&self) -> Self::AccFlags {
        Self::AccFlags::new()
    }
}
