use core::{iter::Chain, slice};

use generic_array_struct::generic_array_struct;

use crate::{instructions::internal_utils::caba, traits::SolValCalcAccs};

pub mod lst_to_sol;
pub mod sol_to_lst;

mod internal_utils;

// Accounts

#[generic_array_struct(builder pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct IxPreAccs<T> {
    pub lst_mint: T,
}

impl<T> IxPreAccs<T> {
    #[inline]
    pub const fn memset(val: T) -> Self {
        Self([val; IX_PRE_ACCS_LEN])
    }
}

pub type IxPreKeys<'a> = IxPreAccs<&'a [u8; 32]>;

pub type IxPreKeysOwned = IxPreAccs<[u8; 32]>;

pub type IxPreAccFlags = IxPreAccs<bool>;

pub const IX_PRE_IS_WRITER: IxPreAccFlags = IxPreAccFlags::memset(false);

pub const IX_PRE_IS_SIGNER: IxPreAccFlags = IxPreAccFlags::memset(false);

impl IxPreKeys<'_> {
    #[inline]
    pub fn into_owned(&self) -> IxPreKeysOwned {
        IxPreAccs(self.0.map(|p| *p))
    }
}

impl IxPreKeysOwned {
    #[inline]
    pub fn as_borrowed(&self) -> IxPreKeys<'_> {
        IxPreAccs(self.0.each_ref())
    }
}

// Data

pub const IX_DATA_LEN: usize = 9;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IxData<const DISCM: u8>([u8; IX_DATA_LEN]);

impl<const DISCM: u8> IxData<DISCM> {
    /// # Args
    /// - `amt`. This is LST amount for `LstToSol` and lamport amount for `SolToLst`
    #[inline]
    pub const fn new(amt: u64) -> Self {
        const A: usize = IX_DATA_LEN;

        let mut d = [0u8; A];
        d = caba::<A, 0, 1>(d, &[DISCM]);
        d = caba::<A, 1, 8>(d, &amt.to_le_bytes());

        Self(d)
    }

    #[inline]
    pub const fn as_buf(&self) -> &[u8; IX_DATA_LEN] {
        &self.0
    }
}

// Genericized Input

pub struct IxAccs<T, P> {
    /// Interface account prefix; [`IxPreAccs`]
    pub ix_prefix: IxPreAccs<T>,

    /// Account suffix specific to each implementation
    pub suf: P,
}

impl<T, P> IxAccs<T, P> {
    /// For more convenient usage with type aliases
    #[inline]
    pub const fn new(ix_prefix: IxPreAccs<T>, suf: P) -> Self {
        Self { ix_prefix, suf }
    }
}

pub type AccsIter<'a, T> = Chain<slice::Iter<'a, T>, slice::Iter<'a, T>>;

impl<T, P: AsRef<[T]>> IxAccs<T, P> {
    #[inline]
    pub fn seq(&self) -> AccsIter<'_, T> {
        let Self { ix_prefix, suf } = self;
        ix_prefix.0.iter().chain(suf.as_ref().iter())
    }
}

// Combined accs

/// Call [`IxAccs::seq`] on return value to create iterator
pub fn svc_ix_keys_owned<S: SolValCalcAccs>(
    IxAccs { ix_prefix, suf }: &IxAccs<[u8; 32], S>,
) -> IxAccs<[u8; 32], S::KeysOwned> {
    IxAccs {
        ix_prefix: *ix_prefix,
        suf: suf.suf_keys_owned(),
    }
}

/// Call [`IxAccs::seq`] on return value to create iterator
pub fn svc_ix_is_signer<T, S: SolValCalcAccs>(
    IxAccs { ix_prefix: _, suf }: &IxAccs<T, S>,
) -> IxAccs<bool, S::AccFlags> {
    IxAccs {
        ix_prefix: IX_PRE_IS_SIGNER,
        suf: suf.suf_is_signer(),
    }
}

/// Call [`IxAccs::seq`] on return value to create iterator
pub fn svc_ix_is_writer<T, S: SolValCalcAccs>(
    IxAccs { ix_prefix: _, suf }: &IxAccs<T, S>,
) -> IxAccs<bool, S::AccFlags> {
    IxAccs {
        ix_prefix: IX_PRE_IS_WRITER,
        suf: suf.suf_is_writer(),
    }
}
