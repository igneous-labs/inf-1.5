use inf1_pricing_core::traits::PriceExactOutAccs;

use super::{IxSufAccs, IX_SUF_IS_SIGNER, IX_SUF_IS_WRITER};

pub type PriceExactOutIxSufAccs<T> = IxSufAccs<T>;

pub type PriceExactOutIxSufKeys<'a> = PriceExactOutIxSufAccs<&'a [u8; 32]>;

pub type PriceExactOutIxSufKeysOwned = PriceExactOutIxSufAccs<[u8; 32]>;

pub type PriceExactOutIxSufAccFlags = PriceExactOutIxSufAccs<bool>;

pub const PRICE_EXACT_OUT_IX_SUF_IS_WRITER: PriceExactOutIxSufAccFlags = IX_SUF_IS_WRITER;

pub const PRICE_EXACT_OUT_IX_SUF_IS_SIGNER: PriceExactOutIxSufAccFlags = IX_SUF_IS_SIGNER;

impl PriceExactOutAccs for PriceExactOutIxSufKeysOwned {
    type KeysOwned = Self;
    type AccFlags = PriceExactOutIxSufAccFlags;

    #[inline]
    fn suf_keys_owned(&self) -> Self::KeysOwned {
        *self
    }

    #[inline]
    fn suf_is_writer(&self) -> Self::AccFlags {
        PRICE_EXACT_OUT_IX_SUF_IS_WRITER
    }

    #[inline]
    fn suf_is_signer(&self) -> Self::AccFlags {
        PRICE_EXACT_OUT_IX_SUF_IS_SIGNER
    }
}
