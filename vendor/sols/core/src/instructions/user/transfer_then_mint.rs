use crate::instructions::{
    common::{TokenSysProgs, TOKEN_SYS_PROG_IS_SIGNER, TOKEN_SYS_PROG_IS_WRITER},
    internal_utils::{csi_at, U64OptIxData},
    user::MintClaimCoreAccs,
};

// Accounts

pub const TTM_IX_PRE_IS_WRITER: MintClaimCoreAccs<bool> = MintClaimCoreAccs::memset(true);

pub const TTM_IX_PRE_IS_SIGNER: MintClaimCoreAccs<bool> = MintClaimCoreAccs::memset(false);

/// TransferThenMint
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TTMIxAccs<P, T, X> {
    /// [`MintClaimCoreAccs`]
    pub pre: P,

    /// The system account performing the transfer
    pub inp: T,

    /// [`TokenSysProgs`]
    pub progs: X,
}

pub const TTM_IX_INP_IS_WRITER: bool = true;

pub const TTM_IX_INP_IS_SIGNER: bool = true;

pub type TTMIxGen<T> = TTMIxAccs<MintClaimCoreAccs<T>, T, TokenSysProgs<T>>;

pub type TTMIxKeys<'a> = TTMIxGen<&'a [u8; 32]>;
pub type TTMIxKeysOwned = TTMIxGen<[u8; 32]>;
pub type TTMIxAccFlags = TTMIxGen<bool>;

pub const TTM_IX_IS_WRITER: TTMIxAccFlags = TTMIxAccFlags {
    pre: TTM_IX_PRE_IS_WRITER,
    inp: TTM_IX_INP_IS_WRITER,
    progs: TOKEN_SYS_PROG_IS_WRITER,
};

pub const TTM_IX_IS_SIGNER: TTMIxAccFlags = TTMIxAccFlags {
    pre: TTM_IX_PRE_IS_SIGNER,
    inp: TTM_IX_INP_IS_SIGNER,
    progs: TOKEN_SYS_PROG_IS_SIGNER,
};

pub type TTMIxAccsIter<'a, T> = csi_at!(@ @);

impl<T> TTMIxGen<T> {
    #[inline]
    pub fn seq(&self) -> TTMIxAccsIter<'_, T> {
        self.pre
            .0
            .iter()
            .chain(core::slice::from_ref(&self.inp))
            .chain(self.progs.0.iter())
    }
}

// Data

pub const TTM_IX_DISCM: u8 = 1;

pub const TTM_IX_DATA_LEN: usize = TTMIxData::LEN;

pub type TTMIxData = U64OptIxData<TTM_IX_DISCM>;
