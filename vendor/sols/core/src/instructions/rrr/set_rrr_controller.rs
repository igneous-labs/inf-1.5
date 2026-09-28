use crate::instructions::internal_utils::DiscmOnlyIxData;

// Accounts: just SetAuthIxAccs

// Data

pub const SET_RRR_CONTROLLER_IX_DISCM: u8 = 19;

pub const SET_RRR_CONTROLLER_IX_DATA_LEN: usize = SetRRRControllerIxData::LEN;

pub type SetRRRControllerIxData = DiscmOnlyIxData<SET_RRR_CONTROLLER_IX_DISCM>;
