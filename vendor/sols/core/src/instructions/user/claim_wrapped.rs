use crate::instructions::{
    internal_utils::{csi_at, U64OptIxData},
    user::{
        MintClaimCoreAccs, WrappedConstAccs, CLAIM_IX_IS_SIGNER, CLAIM_IX_IS_WRITER,
        WRAPPED_CONST_IS_SIGNER, WRAPPED_CONST_IS_WRITER,
    },
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClaimWrappedIxAccs<P, W> {
    /// [`MintClaimCoreAccs`]
    pub pre: P,

    /// [`WrappedConstAccs`]
    pub wrapped: W,
}

pub type ClaimWrappedIxGen<T> = ClaimWrappedIxAccs<MintClaimCoreAccs<T>, WrappedConstAccs<T>>;

pub type ClaimWrappedIxKeys<'a> = ClaimWrappedIxGen<&'a [u8; 32]>;
pub type ClaimWrappedIxKeysOwned = ClaimWrappedIxGen<[u8; 32]>;
pub type ClaimWrappedIxAccFlags = ClaimWrappedIxGen<bool>;

pub const CLAIM_WRAPPED_IX_IS_WRITER: ClaimWrappedIxAccFlags = ClaimWrappedIxAccFlags {
    pre: CLAIM_IX_IS_WRITER,
    wrapped: WRAPPED_CONST_IS_WRITER,
};

pub const CLAIM_WRAPPED_IX_IS_SIGNER: ClaimWrappedIxAccFlags = ClaimWrappedIxAccFlags {
    pre: CLAIM_IX_IS_SIGNER,
    wrapped: WRAPPED_CONST_IS_SIGNER,
};

pub type ClaimWrappedIxAccsIter<'a, T> = csi_at!(@);

impl<T> ClaimWrappedIxGen<T> {
    #[inline]
    pub fn seq(&self) -> ClaimWrappedIxAccsIter<'_, T> {
        self.pre.0.iter().chain(self.wrapped.0.iter())
    }
}

// Data

pub const CLAIM_WRAPPED_IX_DISCM: u8 = 4;

pub const CLAIM_WRAPPED_IX_DATA_LEN: usize = ClaimWrappedIxData::LEN;

pub type ClaimWrappedIxData = U64OptIxData<CLAIM_WRAPPED_IX_DISCM>;
