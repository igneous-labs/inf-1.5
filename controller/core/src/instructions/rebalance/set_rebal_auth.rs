use generic_array_struct::generic_array_struct;

use crate::instructions::generic::DiscmOnlyIxData;

// Accounts

#[generic_array_struct(builder pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct SetRebalAuthIxAccs<T> {
    /// The signer setting the new rebalance auth.
    /// Can either be pool admin or current rebalance auth.
    pub signer: T,

    /// New rebalance auth to set to
    pub new: T,

    /// The pool's state singleton PDA
    pub pool_state: T,
}

impl<T: Copy> SetRebalAuthIxAccs<T> {
    #[inline]
    pub const fn memset(val: T) -> Self {
        Self([val; SET_REBAL_AUTH_IX_ACCS_LEN])
    }
}

pub type SetRebalAuthIxKeys<'a> = SetRebalAuthIxAccs<&'a [u8; 32]>;

pub type SetRebalAuthIxKeysOwned = SetRebalAuthIxAccs<[u8; 32]>;

pub type SetRebalAuthIxAccFlags = SetRebalAuthIxAccs<bool>;

pub const SET_REBAL_AUTH_IX_IS_WRITER: SetRebalAuthIxAccFlags =
    SetRebalAuthIxAccFlags::memset(false).const_with_pool_state(true);

pub const SET_REBAL_AUTH_IX_IS_SIGNER: SetRebalAuthIxAccFlags =
    SetRebalAuthIxAccFlags::memset(false).const_with_signer(true);

// Data

pub const SET_REBAL_AUTH_IX_DISCM: u8 = 21;

pub type SetRebalAuthIxData = DiscmOnlyIxData<SET_REBAL_AUTH_IX_DISCM>;

pub const SET_REBAL_AUTH_IX_DATA_LEN: usize = SetRebalAuthIxData::DATA_LEN;
