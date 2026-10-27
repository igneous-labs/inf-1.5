use crate::instructions::{
    common::{TOKEN_SYS_PROG_IS_SIGNER, TOKEN_SYS_PROG_IS_WRITER},
    internal_utils::{csi_at, U64OptIxData},
    user::MintClaimCoreAccs,
};

// Accounts

pub const MINT_IX_PRE_IS_WRITER: MintClaimCoreAccs<bool> =
    MintClaimCoreAccs::memset(true).const_with_pool(false);

pub const MINT_IX_PRE_IS_SIGNER: MintClaimCoreAccs<bool> = MintClaimCoreAccs::memset(false);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MintIxAccs<P, T> {
    /// [`MintClaimCoreAccs`]
    pub pre: P,

    /// SPL token program
    ///
    /// Separated out because the program doesn't need to verify this
    pub token: T,
}

pub type MintIxGen<T> = MintIxAccs<MintClaimCoreAccs<T>, T>;

pub type MintIxKeys<'a> = MintIxGen<&'a [u8; 32]>;
pub type MintIxKeysOwned = MintIxGen<[u8; 32]>;
pub type MintIxAccFlags = MintIxGen<bool>;

pub const MINT_IX_IS_WRITER: MintIxAccFlags = MintIxAccFlags {
    pre: MINT_IX_PRE_IS_WRITER,
    token: *TOKEN_SYS_PROG_IS_WRITER.token(),
};

pub const MINT_IX_IS_SIGNER: MintIxAccFlags = MintIxAccFlags {
    pre: MINT_IX_PRE_IS_SIGNER,
    token: *TOKEN_SYS_PROG_IS_SIGNER.token(),
};

pub type MintIxAccsIter<'a, T> = csi_at!(@);

impl<T> MintIxGen<T> {
    #[inline]
    pub fn seq(&self) -> MintIxAccsIter<'_, T> {
        self.pre.0.iter().chain(core::slice::from_ref(&self.token))
    }
}

// Data

pub const MINT_IX_DISCM: u8 = 0;

pub const MINT_IX_DATA_LEN: usize = MintIxData::LEN;

pub type MintIxData = U64OptIxData<MINT_IX_DISCM>;
