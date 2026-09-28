use generic_array_struct::generic_array_struct;

use crate::{
    instructions::{
        common::{
            MintPoolPair, MintPoolPairAccFlags, TOKEN_SYS_PROG_IS_SIGNER, TOKEN_SYS_PROG_IS_WRITER,
        },
        internal_utils::{csi_at, DiscmOnlyIxData},
    },
    internal_utils::impl_memset,
};

// Accounts

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CloseIxAccs<P, S, T> {
    /// [`MintPoolPair`]
    pub pre: P,

    /// [`CloseIxSufAccs`]
    pub suf: S,

    /// SPL token program
    pub token: T,
}

pub type CloseIxGen<T> = CloseIxAccs<MintPoolPair<T>, CloseIxSufAccs<T>, T>;

pub type CloseIxKeys<'a> = CloseIxGen<&'a [u8; 32]>;
pub type CloseIxKeysOwned = CloseIxGen<[u8; 32]>;
pub type CloseIxAccFlags = CloseIxGen<bool>;

pub const CLOSE_IX_PRE_IS_WRITER: MintPoolPairAccFlags = MintPoolPairAccFlags::memset(true);

pub const CLOSE_IX_PRE_IS_SIGNER: MintPoolPairAccFlags = MintPoolPairAccFlags::memset(false);

#[generic_array_struct(all pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct CloseIxSufAccs<T> {
    /// Pool's portfolio acc
    pub portfolio: T,

    /// Pool admin
    pub admin: T,
}

pub type CloseIxSufKeys<'a> = CloseIxSufAccs<&'a [u8; 32]>;
pub type CloseIxSufKeysOwned = CloseIxSufAccs<[u8; 32]>;
pub type CloseIxSufAccFlags = CloseIxSufAccs<bool>;

impl_memset!(CloseIxSufAccs);

pub const CLOSE_IX_SUF_IS_WRITER: CloseIxSufAccFlags = CloseIxSufAccFlags::memset(false);

pub const CLOSE_IX_SUF_IS_SIGNER: CloseIxSufAccFlags =
    CloseIxSufAccFlags::memset(false).const_with_admin(true);

pub const CLOSE_IX_IS_WRITER: CloseIxAccFlags = CloseIxAccFlags {
    pre: CLOSE_IX_PRE_IS_WRITER,
    suf: CLOSE_IX_SUF_IS_WRITER,
    token: *TOKEN_SYS_PROG_IS_WRITER.token(),
};

pub const CLOSE_IX_IS_SIGNER: CloseIxAccFlags = CloseIxAccFlags {
    pre: CLOSE_IX_PRE_IS_SIGNER,
    suf: CLOSE_IX_SUF_IS_SIGNER,
    token: *TOKEN_SYS_PROG_IS_SIGNER.token(),
};

pub type CloseIxAccsIter<'a, T> = csi_at!(@ @);

impl<T> CloseIxGen<T> {
    #[inline]
    pub fn seq(&self) -> CloseIxAccsIter<'_, T> {
        self.pre
            .0
            .iter()
            .chain(self.suf.0.iter())
            .chain(core::slice::from_ref(&self.token))
    }
}

// Data

pub const CLOSE_IX_DISCM: u8 = 10;

pub const CLOSE_IX_DATA_LEN: usize = CloseIxData::LEN;

pub type CloseIxData = DiscmOnlyIxData<CLOSE_IX_DISCM>;
