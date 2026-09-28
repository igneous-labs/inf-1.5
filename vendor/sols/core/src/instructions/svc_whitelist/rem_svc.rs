use crate::instructions::internal_utils::DiscmOnlyIxData;

// Accounts: just super::CrudSvcWhitelistAccs

// Data

pub const REM_SVC_IX_DISCM: u8 = 25;

pub const REM_SVC_IX_DATA_LEN: usize = RemSvcIxData::LEN;

pub type RemSvcIxData = DiscmOnlyIxData<REM_SVC_IX_DISCM>;
