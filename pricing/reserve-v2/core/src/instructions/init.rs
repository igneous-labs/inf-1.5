use generic_array_struct::generic_array_struct;

use super::generic::DiscmOnlyIxData;
use crate::instructions::csi_at;

#[generic_array_struct(all pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct InitIxPreAccs<T> {
    /// The `PricingState` PDA to initialize
    pub pricing_state: T,

    /// The signer paying for the pricing state account's rent
    pub payer: T,
}

impl<T: Copy> InitIxPreAccs<T> {
    #[inline]
    pub const fn memset(val: T) -> Self {
        Self([val; INIT_IX_PRE_ACCS_LEN])
    }
}

impl<T> InitIxPreAccs<T> {
    #[inline]
    pub const fn new(arr: [T; INIT_IX_PRE_ACCS_LEN]) -> Self {
        Self(arr)
    }
}

pub type InitIxPreKeys<'a> = InitIxPreAccs<&'a [u8; 32]>;
pub type InitIxPreKeysOwned = InitIxPreAccs<[u8; 32]>;
pub type InitIxPreAccFlags = InitIxPreAccs<bool>;

pub const INIT_IX_PRE_IS_WRITER: InitIxPreAccFlags = InitIxPreAccFlags::memset(true);

pub const INIT_IX_PRE_IS_SIGNER: InitIxPreAccFlags =
    InitIxPreAccFlags::const_from_destr(InitIxPreAccsDestr {
        payer: true,
        pricing_state: false,
    });

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InitIxAccs<P, T> {
    /// [`InitIxPreAccs`]
    pub pre: P,

    /// system program
    pub sys_prog: T,
}

pub type InitIxAccsGen<T> = InitIxAccs<InitIxPreAccs<T>, T>;

pub const INIT_IX_IS_WRITER: InitIxAccsGen<bool> = InitIxAccs {
    pre: INIT_IX_PRE_IS_WRITER,
    sys_prog: false,
};

pub const INIT_IX_IS_SIGNER: InitIxAccsGen<bool> = InitIxAccs {
    pre: INIT_IX_PRE_IS_SIGNER,
    sys_prog: false,
};

pub type InitAccsIter<'a, T> = csi_at!(@);

impl<T> InitIxAccsGen<T> {
    #[inline]
    pub fn seq(&self) -> InitAccsIter<'_, T> {
        let Self { pre, sys_prog } = self;
        pre.0.iter().chain(core::slice::from_ref(sys_prog).iter())
    }
}

// Data

pub const INIT_IX_DISCM: u8 = 255;

pub type InitIxData = DiscmOnlyIxData<INIT_IX_DISCM>;
