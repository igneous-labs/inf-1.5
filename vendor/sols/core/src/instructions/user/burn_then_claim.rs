use generic_array_struct::generic_array_struct;

use crate::{
    instructions::{
        common::{TOKEN_SYS_PROG_IS_SIGNER, TOKEN_SYS_PROG_IS_WRITER},
        internal_utils::{csi_at, U64OptIxData},
        user::MintClaimCoreAccs,
    },
    internal_utils::impl_memset,
};

// Accounts

/// Also applicable to BurnThenClaimWrapped
pub const BTC_IX_PRE_IS_WRITER: MintClaimCoreAccs<bool> = MintClaimCoreAccs::memset(true);

/// Also applicable to BurnThenClaimWrapped
pub const BTC_IX_PRE_IS_SIGNER: MintClaimCoreAccs<bool> = MintClaimCoreAccs::memset(false);

/// Also applicable to BurnThenClaimWrapped
#[generic_array_struct(all pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct BTCIxSufAccs<T> {
    /// User's SOLS token account to burn SOLS from
    pub inp: T,

    /// User signer
    pub user: T,
}

pub type BTCIxSufKeys<'a> = BTCIxSufAccs<&'a [u8; 32]>;
pub type BTCIxSufKeysOwned = BTCIxSufAccs<[u8; 32]>;
pub type BTCIxSufAccFlags = BTCIxSufAccs<bool>;

impl_memset!(BTCIxSufAccs);

/// Also applicable to BurnThenClaimWrapped
pub const BTC_IX_SUF_IS_WRITER: BTCIxSufAccFlags =
    BTCIxSufAccFlags::const_from_destr(BTCIxSufAccsDestr {
        inp: true,
        user: false,
    });

/// Also applicable to BurnThenClaimWrapped
pub const BTC_IX_SUF_IS_SIGNER: BTCIxSufAccFlags =
    BTCIxSufAccFlags::const_from_destr(BTCIxSufAccsDestr {
        inp: false,
        user: true,
    });

/// BurnThenClaim
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BTCIxAccs<P, S, T> {
    /// [`MintClaimCoreAccs`]
    pub pre: P,

    /// [`BTCIxSufAccs`]
    pub suf: S,

    /// Tokenkeg program
    pub token: T,
}

pub type BTCIxGen<T> = BTCIxAccs<MintClaimCoreAccs<T>, BTCIxSufAccs<T>, T>;

pub type BTCIxKeys<'a> = BTCIxGen<&'a [u8; 32]>;
pub type BTCIxKeysOwned = BTCIxGen<[u8; 32]>;
pub type BTCIxAccFlags = BTCIxGen<bool>;

pub const BTC_IX_IS_WRITER: BTCIxAccFlags = BTCIxAccFlags {
    pre: BTC_IX_PRE_IS_WRITER,
    suf: BTC_IX_SUF_IS_WRITER,
    token: *TOKEN_SYS_PROG_IS_WRITER.token(),
};

pub const BTC_IX_IS_SIGNER: BTCIxAccFlags = BTCIxAccFlags {
    pre: BTC_IX_PRE_IS_SIGNER,
    suf: BTC_IX_SUF_IS_SIGNER,
    token: *TOKEN_SYS_PROG_IS_SIGNER.token(),
};

pub type BTCIxAccsIter<'a, T> = csi_at!(@ @);

impl<T> BTCIxGen<T> {
    #[inline]
    pub fn seq(&self) -> BTCIxAccsIter<'_, T> {
        self.pre
            .0
            .iter()
            .chain(self.suf.0.iter())
            .chain(core::slice::from_ref(&self.token))
    }
}

// Data

pub const BTC_IX_DISCM: u8 = 5;

pub const BTC_IX_DATA_LEN: usize = BTCIxData::LEN;

pub type BTCIxData = U64OptIxData<BTC_IX_DISCM>;
