use generic_array_struct::generic_array_struct;

use crate::instructions::{
    admin::common::{ADMIN_IX_PRE_IS_SIGNER, ADMIN_IX_PRE_IS_WRITER},
    csi_at,
};

use super::common::AdminIxPreAccs;

#[generic_array_struct(all pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct SetFeeEntryIxSufAccs<T> {
    /// accepted SPL token mint to set
    pub mint: T,

    /// funds account growth if needed
    pub payer: T,
}

impl<T: Copy> SetFeeEntryIxSufAccs<T> {
    #[inline]
    pub const fn memset(val: T) -> Self {
        Self([val; SET_FEE_ENTRY_IX_SUF_ACCS_LEN])
    }
}

impl<T> SetFeeEntryIxSufAccs<T> {
    #[inline]
    pub const fn new(arr: [T; SET_FEE_ENTRY_IX_SUF_ACCS_LEN]) -> Self {
        Self(arr)
    }
}

pub type SetFeeEntryIxSufKeys<'a> = SetFeeEntryIxSufAccs<&'a [u8; 32]>;
pub type SetFeeEntryIxSufKeysOwned = SetFeeEntryIxSufAccs<[u8; 32]>;
pub type SetFeeEntryIxSufAccFlags = SetFeeEntryIxSufAccs<bool>;

pub const SET_FEE_ENTRY_IX_SUF_IS_WRITER: SetFeeEntryIxSufAccFlags =
    SetFeeEntryIxSufAccFlags::const_from_destr(SetFeeEntryIxSufAccsDestr {
        mint: false,
        payer: true,
    });

pub const SET_FEE_ENTRY_IX_SUF_IS_SIGNER: SetFeeEntryIxSufAccFlags =
    SetFeeEntryIxSufAccFlags::const_from_destr(SetFeeEntryIxSufAccsDestr {
        mint: false,
        payer: true,
    });

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SetFeeEntryIxAccs<P, S, T> {
    /// [`AdminIxPreAccs`]
    pub pre: P,

    /// [`SetFeeEntryIxSufAccs`]
    pub suf: S,

    /// system program
    pub sys_prog: T,
}

pub type SetFeeEntryIxAccsGen<T> = SetFeeEntryIxAccs<AdminIxPreAccs<T>, SetFeeEntryIxSufAccs<T>, T>;

pub const SET_FEE_ENTRY_IX_IS_WRITER: SetFeeEntryIxAccsGen<bool> = SetFeeEntryIxAccs {
    pre: ADMIN_IX_PRE_IS_WRITER,
    suf: SET_FEE_ENTRY_IX_SUF_IS_WRITER,
    sys_prog: false,
};

pub const SET_FEE_ENTRY_IX_IS_SIGNER: SetFeeEntryIxAccsGen<bool> = SetFeeEntryIxAccs {
    pre: ADMIN_IX_PRE_IS_SIGNER,
    suf: SET_FEE_ENTRY_IX_SUF_IS_SIGNER,
    sys_prog: false,
};

pub type SetFeeEntryAccsIter<'a, T> = csi_at!(@ @);

impl<T> SetFeeEntryIxAccsGen<T> {
    #[inline]
    pub fn seq(&self) -> SetFeeEntryAccsIter<'_, T> {
        let Self { pre, suf, sys_prog } = self;
        pre.0
            .iter()
            .chain(suf.0.iter())
            .chain(core::slice::from_ref(sys_prog).iter())
    }
}

// Data — TODO
