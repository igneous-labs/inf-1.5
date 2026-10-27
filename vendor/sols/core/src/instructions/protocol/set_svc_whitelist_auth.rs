use crate::instructions::internal_utils::DiscmOnlyIxData;

// Accounts: just SetAuthIxAccs

// Data

pub const SET_SWA_IX_DISCM: u8 = 23;

pub const SET_SWA_IX_DATA_LEN: usize = SetSWAIxData::LEN;

pub type SetSWAIxData = DiscmOnlyIxData<SET_SWA_IX_DISCM>;
