use generic_array_struct::generic_array_struct;

use crate::instructions::internal_utils::DiscmOnlyIxData;

// Accounts

#[generic_array_struct(builder pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct EnablePoolIxAccs<T> {
    /// pool admin
    pub admin: T,

    /// The pool's state singleton PDA
    pub pool_state: T,
}

impl<T: Copy> EnablePoolIxAccs<T> {
    #[inline]
    pub const fn memset(val: T) -> Self {
        Self([val; ENABLE_POOL_IX_ACCS_LEN])
    }
}

pub type EnablePoolIxKeys<'a> = EnablePoolIxAccs<&'a [u8; 32]>;

pub type EnablePoolIxKeysOwned = EnablePoolIxAccs<[u8; 32]>;

pub type EnablePoolIxAccFlags = EnablePoolIxAccs<bool>;

pub const ENABLE_POOL_IX_IS_WRITER: EnablePoolIxAccFlags =
    EnablePoolIxAccFlags::memset(false).const_with_pool_state(true);

pub const ENABLE_POOL_IX_IS_SIGNER: EnablePoolIxAccFlags =
    EnablePoolIxAccFlags::memset(false).const_with_admin(true);

// Data

pub const ENABLE_POOL_IX_DISCM: u8 = 18;

pub type EnablePoolIxData = DiscmOnlyIxData<ENABLE_POOL_IX_DISCM>;

pub const ENABLE_POOL_IX_DATA_LEN: usize = EnablePoolIxData::DATA_LEN;
