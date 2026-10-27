use generic_array_struct::generic_array_struct;

use crate::{
    instructions::{
        common::{TokenSysProgs, TOKEN_SYS_PROG_IS_SIGNER, TOKEN_SYS_PROG_IS_WRITER},
        internal_utils::{csi_at, U64OptIxData},
        user::{MintClaimCoreAccs, TTM_IX_PRE_IS_SIGNER, TTM_IX_PRE_IS_WRITER},
    },
    internal_utils::impl_memset,
};

// Accounts

pub const TWTM_IX_PRE_IS_WRITER: MintClaimCoreAccs<bool> = TTM_IX_PRE_IS_WRITER;

pub const TWTM_IX_PRE_IS_SIGNER: MintClaimCoreAccs<bool> = TTM_IX_PRE_IS_SIGNER;

#[generic_array_struct(all pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct TWTMIxSufAccs<T> {
    /// User's wSOL token account to transfer wSOL from
    pub inp: T,

    /// The pool's wSOL bridge token acc
    pub wsol_bridge: T,

    /// User signer
    pub user: T,

    /// wSOL mint
    pub wsol_mint: T,
}

pub type TWTMIxSufKeys<'a> = TWTMIxSufAccs<&'a [u8; 32]>;
pub type TWTMIxSufKeysOwned = TWTMIxSufAccs<[u8; 32]>;
pub type TWTMIxSufAccFlags = TWTMIxSufAccs<bool>;

impl_memset!(TWTMIxSufAccs);

impl<T> TWTMIxSufAccs<T> {
    /// SPL token instruction: Transfer from `inp` to `wsol_bridge`, authorized by `user`
    #[inline]
    pub const fn as_transfer_token_ix(&self) -> &[T; 3] {
        match self.0.first_chunk() {
            Some(x) => x,
            None => unreachable!(),
        }
    }
}

pub const TWTM_IX_SUF_IS_WRITER: TWTMIxSufAccFlags = TWTMIxSufAccFlags::memset(false)
    .const_with_inp(true)
    .const_with_wsol_bridge(true);

pub const TWTM_IX_SUF_IS_SIGNER: TWTMIxSufAccFlags =
    TWTMIxSufAccFlags::memset(false).const_with_user(true);

/// TransferWrappedThenMint
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TWTMIxAccs<P, S, X> {
    /// [`MintClaimCoreAccs`]
    pub pre: P,

    /// [`TWTMIxSufAccs`]
    pub suf: S,

    /// [`TokenSysProgs`]
    pub progs: X,
}

pub type TWTMIxGen<T> = TWTMIxAccs<MintClaimCoreAccs<T>, TWTMIxSufAccs<T>, TokenSysProgs<T>>;

pub type TWTMIxKeys<'a> = TWTMIxGen<&'a [u8; 32]>;
pub type TWTMIxKeysOwned = TWTMIxGen<[u8; 32]>;
pub type TWTMIxAccFlags = TWTMIxGen<bool>;

pub const TWTM_IX_IS_WRITER: TWTMIxAccFlags = TWTMIxAccFlags {
    pre: TWTM_IX_PRE_IS_WRITER,
    suf: TWTM_IX_SUF_IS_WRITER,
    progs: TOKEN_SYS_PROG_IS_WRITER,
};

pub const TWTM_IX_IS_SIGNER: TWTMIxAccFlags = TWTMIxAccFlags {
    pre: TWTM_IX_PRE_IS_SIGNER,
    suf: TWTM_IX_SUF_IS_SIGNER,
    progs: TOKEN_SYS_PROG_IS_SIGNER,
};

pub type TWTMIxAccsIter<'a, T> = csi_at!(@ @);

impl<T> TWTMIxGen<T> {
    #[inline]
    pub fn seq(&self) -> TWTMIxAccsIter<'_, T> {
        self.pre
            .0
            .iter()
            .chain(self.suf.0.iter())
            .chain(self.progs.0.iter())
    }
}

// Data

pub const TWTM_IX_DISCM: u8 = 2;

pub const TWTM_IX_DATA_LEN: usize = TWTMIxData::LEN;

pub type TWTMIxData = U64OptIxData<TWTM_IX_DISCM>;
