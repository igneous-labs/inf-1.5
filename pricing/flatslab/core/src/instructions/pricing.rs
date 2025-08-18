// All 4 pricing program interface instructions just have the same account suffix:
// just the slab PDA

use generic_array_struct::generic_array_struct;
use inf1_pp_core::traits::main::{PriceExactInAccs, PriceExactOutAccs};

#[allow(deprecated)]
use inf1_pp_core::traits::deprecated::{PriceLpTokensToMintAccs, PriceLpTokensToRedeemAccs};

use crate::keys::SLAB_ID;

#[generic_array_struct(builder pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct IxSufAccs<T> {
    /// The slab PDA
    pub slab: T,
}

impl<T> IxSufAccs<T> {
    #[inline]
    pub const fn memset(v: T) -> Self {
        Self([v; IX_SUF_ACCS_LEN])
    }

    /// For more convenient usage with type aliases
    #[inline]
    pub const fn new(arr: [T; IX_SUF_ACCS_LEN]) -> Self {
        Self(arr)
    }
}

impl<T> AsRef<[T]> for IxSufAccs<T> {
    #[inline]
    fn as_ref(&self) -> &[T] {
        &self.0
    }
}

pub type IxSufKeys<'a> = IxSufAccs<&'a [u8; 32]>;

pub type IxSufKeysOwned = IxSufAccs<[u8; 32]>;

pub type IxSufAccFlags = IxSufAccs<bool>;

pub const IX_SUF_IS_WRITER: IxSufAccFlags = IxSufAccFlags::memset(false);

pub const IX_SUF_IS_SIGNER: IxSufAccFlags = IxSufAccFlags::memset(false);

// simple newtype so that the *KeysOwned struct doesnt implement pricing prog accs trait directly
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct FlatSlabPpAccs(pub IxSufKeysOwned);

impl FlatSlabPpAccs {
    pub const MAINNET: Self = Self(IxSufKeysOwned::memset(SLAB_ID));

    #[inline]
    pub const fn new(slab: [u8; 32]) -> Self {
        Self(IxSufAccs([slab]))
    }
}

impl FlatSlabPpAccs {
    #[inline]
    pub const fn pp_suf_keys_owned(&self) -> IxSufKeysOwned {
        self.0
    }

    #[inline]
    pub const fn pp_suf_is_writer(&self) -> IxSufAccFlags {
        IX_SUF_IS_WRITER
    }

    #[inline]
    pub const fn pp_suf_is_signer(&self) -> IxSufAccFlags {
        IX_SUF_IS_SIGNER
    }
}

macro_rules! impl_pricing_trait {
    ($Trait:ty) => {
        #[allow(deprecated)]
        impl $Trait for FlatSlabPpAccs {
            type KeysOwned = IxSufKeysOwned;
            type AccFlags = IxSufAccFlags;

            #[inline]
            fn suf_keys_owned(&self) -> Self::KeysOwned {
                self.pp_suf_keys_owned()
            }

            #[inline]
            fn suf_is_writer(&self) -> Self::AccFlags {
                self.pp_suf_is_writer()
            }

            #[inline]
            fn suf_is_signer(&self) -> Self::AccFlags {
                self.pp_suf_is_signer()
            }
        }
    };
}

pub type PriceExactInIxSufKeysOwned = IxSufKeysOwned;
pub type PriceExactInIxSufAccFlags = IxSufAccFlags;
impl_pricing_trait!(PriceExactInAccs);

pub type PriceExactOutIxSufKeysOwned = IxSufKeysOwned;
pub type PriceExactOutIxSufAccFlags = IxSufAccFlags;
impl_pricing_trait!(PriceExactOutAccs);

pub type PriceLpTokensToMintIxSufKeysOwned = IxSufKeysOwned;
pub type PriceLpTokensToMintIxSufAccFlags = IxSufAccFlags;
impl_pricing_trait!(PriceLpTokensToMintAccs);

pub type PriceLpTokensToRedeemIxSufKeysOwned = IxSufKeysOwned;
pub type PriceLpTokensToRedeemIxSufAccFlags = IxSufAccFlags;
impl_pricing_trait!(PriceLpTokensToRedeemAccs);
