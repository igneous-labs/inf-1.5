use inf1_pricing_core::traits::PriceExactInAccs;

use super::{FlatFeePriceAccs, IxSufAccs, IX_SUF_IS_SIGNER, IX_SUF_IS_WRITER};

pub type PriceExactInIxSufAccs<T> = IxSufAccs<T>;

pub type PriceExactInIxSufKeys<'a> = PriceExactInIxSufAccs<&'a [u8; 32]>;

pub type PriceExactInIxSufKeysOwned = PriceExactInIxSufAccs<[u8; 32]>;

pub type PriceExactInIxSufAccFlags = PriceExactInIxSufAccs<bool>;

pub const PRICE_EXACT_IN_IX_SUF_IS_WRITER: PriceExactInIxSufAccFlags = IX_SUF_IS_WRITER;

pub const PRICE_EXACT_IN_IX_SUF_IS_SIGNER: PriceExactInIxSufAccFlags = IX_SUF_IS_SIGNER;

impl PriceExactInAccs for FlatFeePriceAccs {
    type KeysOwned = PriceExactInIxSufKeysOwned;
    type AccFlags = PriceExactInIxSufAccFlags;

    #[inline]
    fn suf_keys_owned(&self) -> Self::KeysOwned {
        self.pp_price_suf_keys_owned()
    }

    #[inline]
    fn suf_is_writer(&self) -> Self::AccFlags {
        self.pp_price_suf_is_writer()
    }

    #[inline]
    fn suf_is_signer(&self) -> Self::AccFlags {
        self.pp_price_suf_is_signer()
    }
}
