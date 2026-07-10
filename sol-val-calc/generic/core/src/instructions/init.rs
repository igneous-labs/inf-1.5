use core::{
    iter::{once, Chain, Once},
    slice,
};

use generic_array_struct::generic_array_struct;

use crate::instructions::discm_only::DiscmOnlyIxData;

// Accounts

#[generic_array_struct(all pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct InitIxPreAccs<T> {
    /// Signer funding `state`
    pub payer: T,

    /// State PDA to be created
    pub state: T,
}

impl<T: Copy> InitIxPreAccs<T> {
    #[inline]
    pub const fn memset(val: T) -> Self {
        Self([val; INIT_IX_PRE_ACCS_LEN])
    }
}

pub type InitIxPreKeys<'a> = InitIxPreAccs<&'a [u8; 32]>;

pub type InitIxPreKeysOwned = InitIxPreAccs<[u8; 32]>;

pub type InitIxPreAccFlags = InitIxPreAccs<bool>;

impl<T> AsRef<[T]> for InitIxPreAccs<T> {
    #[inline]
    fn as_ref(&self) -> &[T] {
        &self.0
    }
}

pub const INIT_IX_PRE_IS_WRITER: InitIxPreAccFlags = InitIxPreAccFlags::memset(true);

pub const INIT_IX_PRE_IS_SIGNER: InitIxPreAccFlags =
    InitIxPreAccs::const_from_destr(InitIxPreAccsDestr {
        payer: true,
        state: false,
    });

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InitIxAccs<P, T> {
    /// [`InitIxPreAccs`]
    pub pre: P,

    pub sys_prog: T,
}

pub type InitIxAccsGen<T> = InitIxAccs<InitIxPreAccs<T>, T>;

pub type InitIxKeys<'a> = InitIxAccsGen<&'a [u8; 32]>;
pub type InitIxKeysOwned = InitIxAccsGen<[u8; 32]>;
pub type InitIxAccFlags = InitIxAccsGen<bool>;

pub type InitIxAccsIter<'a, T> = Chain<slice::Iter<'a, T>, Once<&'a T>>;

impl<P: AsRef<[T]>, T> InitIxAccs<P, T> {
    #[inline]
    pub fn seq(&self) -> InitIxAccsIter<'_, T> {
        let Self { pre, sys_prog } = self;
        pre.as_ref().iter().chain(once(sys_prog))
    }
}

pub const INIT_IX_IS_WRITER: InitIxAccFlags = InitIxAccs {
    pre: INIT_IX_PRE_IS_WRITER,
    sys_prog: false,
};

pub const INIT_IX_IS_SIGNER: InitIxAccsGen<bool> = InitIxAccs {
    pre: INIT_IX_PRE_IS_SIGNER,
    sys_prog: false,
};

// Data

pub const INIT_IX_DISCM: u8 = 255;

pub const INIT_IX_DATA_LEN: usize = InitIxData::DATA_LEN;

pub type InitIxData = DiscmOnlyIxData<INIT_IX_DISCM>;
