use generic_array_struct::generic_array_struct;

use super::common::AdminIxPreAccs;
use crate::instructions::{
    admin::common::{ADMIN_IX_PRE_IS_SIGNER, ADMIN_IX_PRE_IS_WRITER},
    csi_at,
    generic::DiscmOnlyIxData,
};

// Suf accs — instruction-specific accounts

#[generic_array_struct(all pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct RemoveFeeEntryIxSufAccs<T> {
    /// mint to remove
    pub mint: T,

    /// receives excess rent from account shrinking
    pub refund_rent_to: T,
}

impl<T: Copy> RemoveFeeEntryIxSufAccs<T> {
    #[inline]
    pub const fn memset(val: T) -> Self {
        Self([val; REMOVE_FEE_ENTRY_IX_SUF_ACCS_LEN])
    }
}

impl<T> RemoveFeeEntryIxSufAccs<T> {
    #[inline]
    pub const fn new(arr: [T; REMOVE_FEE_ENTRY_IX_SUF_ACCS_LEN]) -> Self {
        Self(arr)
    }
}

pub type RemoveFeeEntryIxSufKeys<'a> = RemoveFeeEntryIxSufAccs<&'a [u8; 32]>;
pub type RemoveFeeEntryIxSufKeysOwned = RemoveFeeEntryIxSufAccs<[u8; 32]>;
pub type RemoveFeeEntryIxSufAccFlags = RemoveFeeEntryIxSufAccs<bool>;

pub const REMOVE_FEE_ENTRY_IX_SUF_IS_WRITER: RemoveFeeEntryIxSufAccFlags =
    RemoveFeeEntryIxSufAccFlags::const_from_destr(RemoveFeeEntryIxSufAccsDestr {
        mint: false,
        refund_rent_to: true,
    });

pub const REMOVE_FEE_ENTRY_IX_SUF_IS_SIGNER: RemoveFeeEntryIxSufAccFlags =
    RemoveFeeEntryIxSufAccFlags::memset(false);

// Composite

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RemoveFeeEntryIxAccs<P, T> {
    /// [`AdminIxPreAccs`]
    pub pre: P,

    /// [`RemoveFeeEntryIxSufAccs`]
    pub suf: T,
}

pub type RemoveFeeEntryIxAccsGen<T> =
    RemoveFeeEntryIxAccs<AdminIxPreAccs<T>, RemoveFeeEntryIxSufAccs<T>>;

pub const REMOVE_FEE_ENTRY_IX_IS_WRITER: RemoveFeeEntryIxAccsGen<bool> = RemoveFeeEntryIxAccs {
    pre: ADMIN_IX_PRE_IS_WRITER,
    suf: REMOVE_FEE_ENTRY_IX_SUF_IS_WRITER,
};

pub const REMOVE_FEE_ENTRY_IX_IS_SIGNER: RemoveFeeEntryIxAccsGen<bool> = RemoveFeeEntryIxAccs {
    pre: ADMIN_IX_PRE_IS_SIGNER,
    suf: REMOVE_FEE_ENTRY_IX_SUF_IS_SIGNER,
};

pub type RemoveFeeEntryAccsIter<'a, T> = csi_at!(@);

impl<T> RemoveFeeEntryIxAccsGen<T> {
    #[inline]
    pub fn seq(&self) -> RemoveFeeEntryAccsIter<'_, T> {
        let Self { pre, suf } = self;
        pre.0.iter().chain(suf.0.iter())
    }
}

// Data

pub const REMOVE_FEE_ENTRY_IX_DISCM: u8 = 252;

pub type RemoveFeeEntryIxData = DiscmOnlyIxData<REMOVE_FEE_ENTRY_IX_DISCM>;
