use generic_array_struct::generic_array_struct;

use crate::instructions::generic::U32IxData;

// Accounts

#[generic_array_struct(builder pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct SetProtocolFeeIxAccs<T> {
    /// The pool's admin
    pub admin: T,

    /// The pool's state singleton PDA
    pub pool_state: T,
}

impl<T: Copy> SetProtocolFeeIxAccs<T> {
    #[inline]
    pub const fn memset(val: T) -> Self {
        Self([val; SET_PROTOCOL_FEE_IX_ACCS_LEN])
    }
}

pub type SetProtocolFeeIxKeys<'a> = SetProtocolFeeIxAccs<&'a [u8; 32]>;

pub type SetProtocolFeeIxKeysOwned = SetProtocolFeeIxAccs<[u8; 32]>;

pub type SetProtocolFeeIxAccFlags = SetProtocolFeeIxAccs<bool>;

pub const SET_PROTOCOL_FEE_IX_IS_WRITER: SetProtocolFeeIxAccFlags =
    SetProtocolFeeIxAccFlags::memset(false).const_with_pool_state(true);

pub const SET_PROTOCOL_FEE_IX_IS_SIGNER: SetProtocolFeeIxAccFlags =
    SetProtocolFeeIxAccFlags::memset(false).const_with_admin(true);

// Data

pub const SET_PROTOCOL_FEE_IX_DISCM: u8 = 11;

pub type SetProtocolFeeIxData = U32IxData<SET_PROTOCOL_FEE_IX_DISCM>;

pub const SET_PROTOCOL_FEE_IX_DATA_LEN: usize = SetProtocolFeeIxData::DATA_LEN;
