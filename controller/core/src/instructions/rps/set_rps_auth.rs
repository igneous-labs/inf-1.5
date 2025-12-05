use generic_array_struct::generic_array_struct;

use crate::instructions::generic::DiscmOnlyIxData;

// Accounts

#[generic_array_struct(builder pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct SetRpsAuthIxAccs<T> {
    /// The pool's state singleton PDA
    pub pool_state: T,

    /// The signer setting the new RPS auth.
    /// Can either be pool admin or current RPS authority.
    pub signer: T,

    /// New RPS auth to set to
    pub new_rps_auth: T,
}

impl<T: Copy> SetRpsAuthIxAccs<T> {
    #[inline]
    pub const fn memset(val: T) -> Self {
        Self([val; SET_RPS_AUTH_IX_ACCS_LEN])
    }
}

pub type SetRpsAuthIxKeys<'a> = SetRpsAuthIxAccs<&'a [u8; 32]>;

pub type SetRpsAuthIxKeysOwned = SetRpsAuthIxAccs<[u8; 32]>;

pub type SetRpsAuthIxAccFlags = SetRpsAuthIxAccs<bool>;

pub const SET_RPS_AUTH_IX_IS_WRITER: SetRpsAuthIxAccFlags =
    SetRpsAuthIxAccFlags::memset(false).const_with_pool_state(true);

pub const SET_RPS_AUTH_IX_IS_SIGNER: SetRpsAuthIxAccFlags =
    SetRpsAuthIxAccFlags::memset(false).const_with_signer(true);

// Data

pub const SET_RPS_AUTH_IX_DISCM: u8 = 27;

pub type SetRpsAuthIxData = DiscmOnlyIxData<SET_RPS_AUTH_IX_DISCM>;

pub const SET_RPS_AUTH_IX_DATA_LEN: usize = SetRpsAuthIxData::DATA_LEN;
