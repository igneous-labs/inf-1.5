use generic_array_struct::generic_array_struct;
use inf1_pricing_core::traits::PriceLpTokensToRedeemAccs;

use crate::instructions::internal_utils::impl_asref;

/// This program has no additional accounts suffix
#[generic_array_struct(builder pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct PriceLpTokensToRedeemIxSufAccs<T> {
    /// The pricing program's ProgramState PDA
    pub program_state: T,
}

impl<T> PriceLpTokensToRedeemIxSufAccs<T> {
    #[inline]
    pub const fn memset(v: T) -> Self {
        Self([v])
    }
}

pub type PriceLpTokensToRedeemIxSufKeys<'a> = PriceLpTokensToRedeemIxSufAccs<&'a [u8; 32]>;

pub type PriceLpTokensToRedeemIxSufKeysOwned = PriceLpTokensToRedeemIxSufAccs<[u8; 32]>;

pub type PriceLpTokensToRedeemIxSufAccFlags = PriceLpTokensToRedeemIxSufAccs<bool>;

pub const PRICE_LP_TOKENS_TO_REDEEM_IX_SUF_IS_WRITER: PriceLpTokensToRedeemIxSufAccFlags =
    PriceLpTokensToRedeemIxSufAccFlags::memset(false);

pub const PRICE_LP_TOKENS_TO_REDEEM_IX_SUF_IS_SIGNER: PriceLpTokensToRedeemIxSufAccFlags =
    PriceLpTokensToRedeemIxSufAccFlags::memset(false);

impl_asref!(PriceLpTokensToRedeemIxSufAccs<T>);

impl PriceLpTokensToRedeemAccs for PriceLpTokensToRedeemIxSufKeysOwned {
    type KeysOwned = Self;
    type AccFlags = PriceLpTokensToRedeemIxSufAccFlags;

    #[inline]
    fn suf_keys_owned(&self) -> Self::KeysOwned {
        *self
    }

    #[inline]
    fn suf_is_writer(&self) -> Self::AccFlags {
        PRICE_LP_TOKENS_TO_REDEEM_IX_SUF_IS_WRITER
    }

    #[inline]
    fn suf_is_signer(&self) -> Self::AccFlags {
        PRICE_LP_TOKENS_TO_REDEEM_IX_SUF_IS_SIGNER
    }
}
