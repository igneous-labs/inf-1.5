use generic_array_struct::generic_array_struct;

use crate::instructions::discm_only::DiscmOnlyIxData;

// Accounts

#[generic_array_struct(all pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct SetManagerIxAccs<T> {
    /// The program's current manager
    pub curr: T,

    /// New manager to set to
    pub new: T,

    /// The program's State singleton PDA
    pub state: T,
}

impl<T: Copy> SetManagerIxAccs<T> {
    #[inline]
    pub const fn memset(val: T) -> Self {
        Self([val; SET_MANAGER_IX_ACCS_LEN])
    }
}

pub type SetManagerIxKeys<'a> = SetManagerIxAccs<&'a [u8; 32]>;

pub type SetManagerIxKeysOwned = SetManagerIxAccs<[u8; 32]>;

pub type SetManagerIxAccFlags = SetManagerIxAccs<bool>;

impl<T> AsRef<[T]> for SetManagerIxAccs<T> {
    #[inline]
    fn as_ref(&self) -> &[T] {
        &self.0
    }
}

pub const SET_MANAGER_IX_IS_WRITER: SetManagerIxAccFlags =
    SetManagerIxAccFlags::memset(false).const_with_state(true);

pub const SET_MANAGER_IX_IS_SIGNER: SetManagerIxAccFlags =
    SetManagerIxAccFlags::memset(false).const_with_curr(true);

// Data

pub const SET_MANAGER_IX_DISCM: u8 = 254;

pub const SET_MANAGER_IX_DATA_LEN: usize = SetManagerIxData::DATA_LEN;

pub type SetManagerIxData = DiscmOnlyIxData<SET_MANAGER_IX_DISCM>;
