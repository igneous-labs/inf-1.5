use crate::instructions::internal_utils::DiscmOnlyIxData;

// Accounts: just SetAuthIxAccs

// Data

pub const SET_ADMIN_IX_DISCM: u8 = 11;

pub const SET_ADMIN_IX_DATA_LEN: usize = SetAdminIxData::LEN;

pub type SetAdminIxData = DiscmOnlyIxData<SET_ADMIN_IX_DISCM>;
