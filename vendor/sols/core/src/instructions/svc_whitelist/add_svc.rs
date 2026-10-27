use crate::instructions::internal_utils::DiscmOnlyIxData;

// Accounts: just super::CrudSvcWhitelistAccs

// Data

pub const ADD_SVC_IX_DISCM: u8 = 24;

pub const ADD_SVC_IX_DATA_LEN: usize = AddSvcIxData::LEN;

pub type AddSvcIxData = DiscmOnlyIxData<ADD_SVC_IX_DISCM>;
