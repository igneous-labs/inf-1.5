use core::{iter::Chain, slice};

use generic_array_struct::generic_array_struct;

use crate::{instructions::generic::DiscmOnlyIxData, keys::CONST_KEYS_OWNED};

// Accounts

#[generic_array_struct(all pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct InitIxPreAccs<T> {
    /// Payer for rent of new pool state account
    pub payer: T,

    /// Hardcoded initial admin authorized to call init
    /// and to set all pool authorities to.
    ///
    /// `lp_token_mint's` mint and freeze authority should be set to this
    pub init_admin: T,

    pub pool_state: T,

    /// 0-supply 9-dp mint to set as pool's LP token mint
    pub lp_token_mint: T,
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

pub const INIT_IX_PRE_IS_WRITER: InitIxPreAccFlags =
    InitIxPreAccFlags::const_from_destr(InitIxPreAccsDestr {
        init_admin: false,
        payer: true,
        pool_state: true,
        lp_token_mint: true,
    });

pub const INIT_IX_PRE_IS_SIGNER: InitIxPreAccFlags =
    InitIxPreAccFlags::const_from_destr(InitIxPreAccsDestr {
        payer: true,
        init_admin: true,
        pool_state: false,
        lp_token_mint: false,
    });

#[generic_array_struct(all pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct InitIxProgs<T> {
    pub token: T,
    pub sys: T,
}

impl<T: Copy> InitIxProgs<T> {
    #[inline]
    pub const fn memset(val: T) -> Self {
        Self([val; INIT_IX_PROGS_LEN])
    }
}

pub type InitIxProgsKeys<'a> = InitIxProgs<&'a [u8; 32]>;
pub type InitIxProgsKeysOwned = InitIxProgs<[u8; 32]>;
pub type InitIxProgsAccFlags = InitIxProgs<bool>;

pub const INIT_IX_PROGS_KEYS_OWNED: InitIxProgs<[u8; 32]> =
    InitIxProgs::const_from_destr(InitIxProgsDestr {
        token: *CONST_KEYS_OWNED.tokenkeg(),
        sys: *CONST_KEYS_OWNED.sys_prog(),
    });

pub const INIT_IX_PROGS_IS_WRITER: InitIxProgsAccFlags = InitIxProgs::memset(false);
pub const INIT_IX_PROGS_IS_SIGNER: InitIxProgsAccFlags = InitIxProgs::memset(false);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InitIxAccs<P, Q> {
    /// [`InitIxPreAccs`]
    pub pre: P,

    /// [`InitIxProgs`]
    pub progs: Q,
}

pub type InitIxAccsGen<T> = InitIxAccs<InitIxPreAccs<T>, InitIxProgs<T>>;

pub const INIT_IX_IS_WRITER: InitIxAccsGen<bool> = InitIxAccs {
    pre: INIT_IX_PRE_IS_WRITER,
    progs: INIT_IX_PROGS_IS_WRITER,
};

pub const INIT_IX_IS_SIGNER: InitIxAccsGen<bool> = InitIxAccs {
    pre: INIT_IX_PRE_IS_SIGNER,
    progs: INIT_IX_PROGS_IS_SIGNER,
};

pub type InitAccsIter<'a, T> = Chain<slice::Iter<'a, T>, slice::Iter<'a, T>>;

impl<T> InitIxAccsGen<T> {
    #[inline]
    pub fn seq(&self) -> InitAccsIter<'_, T> {
        let Self { pre, progs } = self;
        pre.0.iter().chain(progs.0.iter())
    }
}

// Data

pub const INIT_IX_DISCM: u8 = 22;

pub type InitIxData = DiscmOnlyIxData<INIT_IX_DISCM>;

pub const INIT_IX_DATA_LEN: usize = InitIxData::DATA_LEN;
