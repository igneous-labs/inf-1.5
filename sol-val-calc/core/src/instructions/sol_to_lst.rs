use crate::instructions::{IxData, IX_DATA_LEN};

use super::{IxPreAccs, IX_PRE_IS_SIGNER, IX_PRE_IS_WRITER};

// Accounts

pub type SolToLstIxPreAccs<T> = IxPreAccs<T>;

pub type SolToLstIxPreKeys<'a> = SolToLstIxPreAccs<&'a [u8; 32]>;

pub type SolToLstIxPreKeysOwned = SolToLstIxPreAccs<[u8; 32]>;

pub type SolToLstIxPreAccFlags = SolToLstIxPreAccs<bool>;

pub const SOL_TO_LST_IX_PRE_IS_WRITER: SolToLstIxPreAccFlags = IX_PRE_IS_WRITER;

pub const SOL_TO_LST_IX_PRE_IS_SIGNER: SolToLstIxPreAccFlags = IX_PRE_IS_SIGNER;

// Data

pub const SOL_TO_LST_IX_DISCM: u8 = 1;

pub const SOL_TO_LST_IX_DATA_LEN: usize = IX_DATA_LEN;

pub type SolToLstIxData = IxData<SOL_TO_LST_IX_DISCM>;
