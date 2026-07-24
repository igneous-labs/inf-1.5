// All 4 pricing program interface instructions have the same account suffix:
// pricing state, Reserve V2 pool state, and the pool's wSOL reserves

use generic_array_struct::generic_array_struct;
use inf1_pp_core::traits::main::{PriceExactInAccs, PriceExactOutAccs};

#[allow(deprecated)]
use inf1_pp_core::traits::deprecated::{PriceLpTokensToMintAccs, PriceLpTokensToRedeemAccs};

use crate::pda::CONST_PDA_KEYS_OWNED;

#[generic_array_struct(all pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct IxSufAccs<T> {
    pub pricing_state: T,
    pub pool_state: T,
    pub wsol_reserves: T,
}

impl<T: Copy> IxSufAccs<T> {
    #[inline]
    pub const fn memset(v: T) -> Self {
        Self([v; IX_SUF_ACCS_LEN])
    }
}

impl<T> IxSufAccs<T> {
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
pub struct ReserveV2PpAccs(pub IxSufKeysOwned);

impl ReserveV2PpAccs {
    pub const MAINNET: Self = Self(IxSufAccs::const_from_destr(IxSufAccsDestr {
        pricing_state: *CONST_PDA_KEYS_OWNED.pricing_state(),
        pool_state: *CONST_PDA_KEYS_OWNED.pool_state(),
        wsol_reserves: *CONST_PDA_KEYS_OWNED.wsol_reserves(),
    }));

    #[inline]
    pub const fn new(
        pricing_state: [u8; 32],
        pool_state: [u8; 32],
        wsol_reserves: [u8; 32],
    ) -> Self {
        Self(IxSufAccs::const_from_destr(IxSufAccsDestr {
            pricing_state,
            pool_state,
            wsol_reserves,
        }))
    }
}

impl ReserveV2PpAccs {
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
        impl $Trait for ReserveV2PpAccs {
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
