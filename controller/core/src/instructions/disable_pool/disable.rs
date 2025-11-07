use generic_array_struct::generic_array_struct;

use crate::instructions::generic::DiscmOnlyIxData;

// Accounts

#[generic_array_struct(builder pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct DisablePoolIxAccs<T> {
    /// Either pool admin or
    /// a disable pool authority
    pub signer: T,

    /// The pool's state singleton PDA
    pub pool_state: T,

    /// The DisablePoolAuthority list singleton PDA
    pub disable_pool_auth_list: T,
}

impl<T: Copy> DisablePoolIxAccs<T> {
    #[inline]
    pub const fn memset(val: T) -> Self {
        Self([val; DISABLE_POOL_IX_ACCS_LEN])
    }
}

pub type DisablePoolIxKeys<'a> = DisablePoolIxAccs<&'a [u8; 32]>;

pub type DisablePoolIxKeysOwned = DisablePoolIxAccs<[u8; 32]>;

pub type DisablePoolIxAccFlags = DisablePoolIxAccs<bool>;

pub const DISABLE_POOL_IX_IS_WRITER: DisablePoolIxAccFlags =
    DisablePoolIxAccFlags::memset(false).const_with_pool_state(true);

pub const DISABLE_POOL_IX_IS_SIGNER: DisablePoolIxAccFlags =
    DisablePoolIxAccFlags::memset(false).const_with_signer(true);

// Data

pub const DISABLE_POOL_IX_DISCM: u8 = 17;

pub type DisablePoolIxData = DiscmOnlyIxData<DISABLE_POOL_IX_DISCM>;

pub const DISABLE_POOL_IX_DATA_LEN: usize = DisablePoolIxData::DATA_LEN;
