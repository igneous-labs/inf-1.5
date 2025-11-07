use generic_array_struct::generic_array_struct;

use crate::instructions::generic::DiscmOnlyIxData;

// Accounts

#[generic_array_struct(builder pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct SetAdminIxAccs<T> {
    /// The pool's current admin
    pub curr: T,

    /// New pool admin to set to
    pub new: T,

    /// The pool's state singleton PDA
    pub pool_state: T,
}

impl<T: Copy> SetAdminIxAccs<T> {
    #[inline]
    pub const fn memset(val: T) -> Self {
        Self([val; SET_ADMIN_IX_ACCS_LEN])
    }
}

pub type SetAdminIxKeys<'a> = SetAdminIxAccs<&'a [u8; 32]>;

pub type SetAdminIxKeysOwned = SetAdminIxAccs<[u8; 32]>;

pub type SetAdminIxAccFlags = SetAdminIxAccs<bool>;

pub const SET_ADMIN_IX_IS_WRITER: SetAdminIxAccFlags =
    SetAdminIxAccFlags::memset(false).const_with_pool_state(true);

pub const SET_ADMIN_IX_IS_SIGNER: SetAdminIxAccFlags =
    SetAdminIxAccFlags::memset(false).const_with_curr(true);

// Data

pub const SET_ADMIN_IX_DISCM: u8 = 10;

pub type SetAdminIxData = DiscmOnlyIxData<SET_ADMIN_IX_DISCM>;

pub const SET_ADMIN_IX_DATA_LEN: usize = SetAdminIxData::DATA_LEN;
