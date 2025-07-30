use generic_array_struct::generic_array_struct;

#[allow(deprecated)]
use inf1_pp_core::traits::deprecated::PriceLpTokensToMintAccs;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FlatFeeMintLpAccs;

impl FlatFeeMintLpAccs {
    #[inline]
    pub const fn pp_mint_suf_keys_owned(&self) -> PriceLpTokensToMintIxSufKeysOwned {
        PriceLpTokensToMintIxSufAccs::new()
    }

    #[inline]
    pub const fn pp_mint_suf_is_writer(&self) -> PriceLpTokensToMintIxSufAccFlags {
        PriceLpTokensToMintIxSufAccs::new()
    }

    #[inline]
    pub const fn pp_mint_suf_is_signer(&self) -> PriceLpTokensToMintIxSufAccFlags {
        PriceLpTokensToMintIxSufAccs::new()
    }
}

#[allow(deprecated)]
impl PriceLpTokensToMintAccs for FlatFeeMintLpAccs {
    type KeysOwned = PriceLpTokensToMintIxSufKeysOwned;
    type AccFlags = PriceLpTokensToMintIxSufAccFlags;

    #[inline]
    fn suf_keys_owned(&self) -> Self::KeysOwned {
        self.pp_mint_suf_keys_owned()
    }

    #[inline]
    fn suf_is_writer(&self) -> Self::AccFlags {
        self.pp_mint_suf_is_writer()
    }

    #[inline]
    fn suf_is_signer(&self) -> Self::AccFlags {
        self.pp_mint_suf_is_signer()
    }
}
