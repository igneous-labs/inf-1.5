use crate::instructions::{
    internal_utils::{csi_at, U64OptIxData},
    user::{
        BTCIxSufAccs, MintClaimCoreAccs, WrappedConstAccs, BTC_IX_PRE_IS_SIGNER,
        BTC_IX_PRE_IS_WRITER, BTC_IX_SUF_IS_SIGNER, BTC_IX_SUF_IS_WRITER, WRAPPED_CONST_IS_SIGNER,
        WRAPPED_CONST_IS_WRITER,
    },
};

// Accounts

/// BurnThenClaimWrapped
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BTCWIxAccs<P, S, W> {
    /// [`MintClaimCoreAccs`]
    pub pre: P,

    /// [`BTCIxSufAccs`]
    pub suf: S,

    /// [`WrappedConstAccs`]
    pub wrapped: W,
}

pub type BTCWIxGen<T> = BTCWIxAccs<MintClaimCoreAccs<T>, BTCIxSufAccs<T>, WrappedConstAccs<T>>;

pub type BTCWIxKeys<'a> = BTCWIxGen<&'a [u8; 32]>;
pub type BTCWIxKeysOwned = BTCWIxGen<[u8; 32]>;
pub type BTCWIxAccFlags = BTCWIxGen<bool>;

pub const BTCW_IX_IS_WRITER: BTCWIxAccFlags = BTCWIxAccFlags {
    pre: BTC_IX_PRE_IS_WRITER,
    suf: BTC_IX_SUF_IS_WRITER,
    wrapped: WRAPPED_CONST_IS_WRITER,
};

pub const BTCW_IX_IS_SIGNER: BTCWIxAccFlags = BTCWIxAccFlags {
    pre: BTC_IX_PRE_IS_SIGNER,
    suf: BTC_IX_SUF_IS_SIGNER,
    wrapped: WRAPPED_CONST_IS_SIGNER,
};

pub type BTCWIxAccsIter<'a, T> = csi_at!(@ @);

impl<T> BTCWIxGen<T> {
    #[inline]
    pub fn seq(&self) -> BTCWIxAccsIter<'_, T> {
        self.pre
            .0
            .iter()
            .chain(self.suf.0.iter())
            .chain(self.wrapped.0.iter())
    }
}

// Data

pub const BTCW_IX_DISCM: u8 = 6;

pub const BTCW_IX_DATA_LEN: usize = BTCWIxData::LEN;

pub type BTCWIxData = U64OptIxData<BTCW_IX_DISCM>;
