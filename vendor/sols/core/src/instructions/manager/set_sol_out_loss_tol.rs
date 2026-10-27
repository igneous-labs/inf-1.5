// Accounts: just SetFieldsIxAccs

// Data

use crate::instructions::internal_utils::NanoIxData;

pub const SET_SOLT_IX_DISCM: u8 = 15;

pub const SET_SOLT_IX_DATA_LEN: usize = SetSOLTIxData::LEN;

pub type SetSOLTIxData = NanoIxData<SET_SOLT_IX_DISCM>;
