use generic_array_struct::generic_array_struct;
use inf1_pp_core::traits::PriceLpTokensToRedeemAccs;

use crate::{instructions::internal_utils::impl_asref, keys::STATE_ID};

#[generic_array_struct(builder pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct FlatFeeRedeemLpAccs(pub PriceLpTokensToRedeemIxSufKeysOwned);

impl FlatFeeRedeemLpAccs {
    pub const MAINNET: Self = Self(PriceLpTokensToRedeemIxSufKeysOwned::memset(STATE_ID));
}

impl Default for FlatFeeRedeemLpAccs {
    #[inline]
    fn default() -> Self {
        Self::MAINNET
    }
}

impl FlatFeeRedeemLpAccs {
    #[inline]
    pub const fn pp_redeem_suf_keys_owned(&self) -> PriceLpTokensToRedeemIxSufKeysOwned {
        self.0
    }

    #[inline]
    pub const fn pp_redeem_suf_is_writer(&self) -> PriceLpTokensToRedeemIxSufAccFlags {
        PRICE_LP_TOKENS_TO_REDEEM_IX_SUF_IS_WRITER
    }

    #[inline]
    pub const fn pp_redeem_suf_is_signer(&self) -> PriceLpTokensToRedeemIxSufAccFlags {
        PRICE_LP_TOKENS_TO_REDEEM_IX_SUF_IS_SIGNER
    }
}

impl PriceLpTokensToRedeemAccs for FlatFeeRedeemLpAccs {
    type KeysOwned = PriceLpTokensToRedeemIxSufKeysOwned;
    type AccFlags = PriceLpTokensToRedeemIxSufAccFlags;

    #[inline]
    fn suf_keys_owned(&self) -> Self::KeysOwned {
        self.pp_redeem_suf_keys_owned()
    }

    #[inline]
    fn suf_is_writer(&self) -> Self::AccFlags {
        self.pp_redeem_suf_is_writer()
    }

    #[inline]
    fn suf_is_signer(&self) -> Self::AccFlags {
        self.pp_redeem_suf_is_signer()
    }
}
