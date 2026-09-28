use crate::instructions::internal_utils::DiscmOnlyIxData;

// Accounts: just SetAuthIxAccs

// Data

pub const SET_PFB_IX_DISCM: u8 = 26;

pub const SET_PFB_IX_DATA_LEN: usize = SetPFBIxData::LEN;

pub type SetPFBIxData = DiscmOnlyIxData<SET_PFB_IX_DISCM>;
