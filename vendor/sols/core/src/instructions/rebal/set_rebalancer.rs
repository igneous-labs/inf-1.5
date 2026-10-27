use crate::instructions::internal_utils::DiscmOnlyIxData;

// Accounts: just SetAuthIxAccs

// Data

pub const SET_REBALANCER_IX_DISCM: u8 = 16;

pub const SET_REBALANCER_IX_DATA_LEN: usize = SetRebalancerIxData::LEN;

pub type SetRebalancerIxData = DiscmOnlyIxData<SET_REBALANCER_IX_DISCM>;
