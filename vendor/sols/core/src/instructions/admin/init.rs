use crate::instructions::{
    common::{
        MintPoolPair, MintPoolPairAccFlags, TokenSysProgs, TOKEN_SYS_PROG_IS_SIGNER,
        TOKEN_SYS_PROG_IS_WRITER,
    },
    internal_utils::{csi_at, DiscmOnlyIxData},
};

// Accounts

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InitIxAccs<P, A, X> {
    /// [`MintPoolPair`]
    pub pre: P,

    /// Pool authority. Other pool authorities are initially set to this as well.
    pub admin: A,

    /// [`TokenSysProgs`]
    pub progs: X,
}

pub type InitIxGen<T> = InitIxAccs<MintPoolPair<T>, T, TokenSysProgs<T>>;

pub type InitIxKeys<'a> = InitIxGen<&'a [u8; 32]>;
pub type InitIxKeysOwned = InitIxGen<[u8; 32]>;
pub type InitIxAccFlags = InitIxGen<bool>;

pub const INIT_IX_PRE_IS_WRITER: MintPoolPairAccFlags = MintPoolPairAccFlags::memset(true);

pub const INIT_IX_PRE_IS_SIGNER: MintPoolPairAccFlags = MintPoolPairAccFlags::memset(false);

pub const INIT_IX_ADMIN_IS_WRITER: bool = false;

pub const INIT_IX_ADMIN_IS_SIGNER: bool = true;

pub const INIT_IX_IS_WRITER: InitIxAccFlags = InitIxAccFlags {
    pre: INIT_IX_PRE_IS_WRITER,
    admin: INIT_IX_ADMIN_IS_WRITER,
    progs: TOKEN_SYS_PROG_IS_WRITER,
};

pub const INIT_IX_IS_SIGNER: InitIxAccFlags = InitIxAccFlags {
    pre: INIT_IX_PRE_IS_SIGNER,
    admin: INIT_IX_ADMIN_IS_SIGNER,
    progs: TOKEN_SYS_PROG_IS_SIGNER,
};

pub type InitIxAccsIter<'a, T> = csi_at!(@ @);

impl<T> InitIxGen<T> {
    #[inline]
    pub fn seq(&self) -> InitIxAccsIter<'_, T> {
        self.pre
            .0
            .iter()
            .chain(core::slice::from_ref(&self.admin))
            .chain(self.progs.0.iter())
    }
}

// Data

pub const INIT_IX_DISCM: u8 = 9;

pub const INIT_IX_DATA_LEN: usize = InitIxData::LEN;

pub type InitIxData = DiscmOnlyIxData<INIT_IX_DISCM>;
