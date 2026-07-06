//! SOL Value Calculator interface instructions

use core::{iter::Chain, slice};

use generic_array_struct::generic_array_struct;
use inf1_svc_core::instructions::{IxPreAccs, IX_PRE_IS_SIGNER, IX_PRE_IS_WRITER};

mod lst_to_sol;
mod sol_to_lst;

pub use lst_to_sol::*;
pub use sol_to_lst::*;

#[generic_array_struct(all pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct IxSufAccs<T> {
    pub state: T,
    pub pool_state: T,
    pub pool_prog: T,
    pub pool_progdata: T,
}

impl<T: Copy> IxSufAccs<T> {
    #[inline]
    pub const fn memset(val: T) -> Self {
        Self([val; IX_SUF_ACCS_LEN])
    }
}

impl<T> AsRef<[T]> for IxSufAccs<T> {
    #[inline]
    fn as_ref(&self) -> &[T] {
        &self.0
    }
}

pub type IxSufKeys<'a> = IxSufAccs<&'a [u8; 32]>;

pub type IxSufKeysOwned = IxSufAccs<[u8; 32]>;

pub type IxSufAccFlags = IxSufAccs<bool>;

pub const IX_SUF_IS_WRITER: IxSufAccFlags = IxSufAccFlags::memset(false);

pub const IX_SUF_IS_SIGNER: IxSufAccFlags = IxSufAccFlags::memset(false);

impl IxSufKeys<'_> {
    #[inline]
    pub fn into_owned(&self) -> IxSufKeysOwned {
        IxSufAccs(self.0.map(|p| *p))
    }
}

impl IxSufKeysOwned {
    #[inline]
    pub fn as_borrowed(&self) -> IxSufKeys<'_> {
        IxSufAccs(self.0.each_ref())
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IxAccs<P, S> {
    /// [`IxPreAccs`]
    pub pre: P,

    /// [`IxSufAccs`]
    pub suf: S,
}

pub type IxAccsGen<T> = IxAccs<IxPreAccs<T>, IxSufAccs<T>>;

pub type IxKeys<'a> = IxAccsGen<&'a [u8; 32]>;
pub type IxKeysOwned = IxAccsGen<[u8; 32]>;
pub type IxAccFlags = IxAccsGen<bool>;

pub type IxAccsIter<'a, T> = Chain<slice::Iter<'a, T>, slice::Iter<'a, T>>;

impl<T> IxAccsGen<T> {
    #[inline]
    pub fn seq(&self) -> IxAccsIter<'_, T> {
        let Self { pre, suf } = self;
        pre.0.iter().chain(suf.0.iter())
    }
}

pub const IX_IS_WRITER: IxAccsGen<bool> = IxAccsGen {
    pre: IX_PRE_IS_WRITER,
    suf: IX_SUF_IS_WRITER,
};

pub const IX_IS_SIGNER: IxAccsGen<bool> = IxAccsGen {
    pre: IX_PRE_IS_SIGNER,
    suf: IX_SUF_IS_SIGNER,
};
