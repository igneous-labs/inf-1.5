use crate::instructions::internal_utils::DiscmOnlyIxData;

// Accounts: just SetAuthIxAccs

// Data

pub const SET_MANAGER_IX_DISCM: u8 = 12;

pub const SET_MANAGER_IX_DATA_LEN: usize = SetManagerIxData::LEN;

pub type SetManagerIxData = DiscmOnlyIxData<SET_MANAGER_IX_DISCM>;
