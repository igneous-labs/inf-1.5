use crate::instructions::internal_utils::DiscmOnlyIxData;

// Accounts: just SetAuthIxAccs

// Data

pub const SET_PFC_IX_DISCM: u8 = 21;

pub const SET_PFC_IX_DATA_LEN: usize = SetPFCIxData::LEN;

pub type SetPFCIxData = DiscmOnlyIxData<SET_PFC_IX_DISCM>;
