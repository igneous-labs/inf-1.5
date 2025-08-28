use crate::instructions::{IxData, IX_DATA_LEN};

use super::{IxPreAccs, IX_PRE_IS_SIGNER, IX_PRE_IS_WRITER};

// Accounts

pub type LstToSolIxPreAccs<T> = IxPreAccs<T>;

pub type LstToSolIxPreKeys<'a> = LstToSolIxPreAccs<&'a [u8; 32]>;

pub type LstToSolIxPreKeysOwned = LstToSolIxPreAccs<[u8; 32]>;

pub type LstToSolIxPreAccFlags = LstToSolIxPreAccs<bool>;

pub const LST_TO_SOL_IX_PRE_IS_WRITER: LstToSolIxPreAccFlags = IX_PRE_IS_WRITER;

pub const LST_TO_SOL_IX_PRE_IS_SIGNER: LstToSolIxPreAccFlags = IX_PRE_IS_SIGNER;

// Data

pub const LST_TO_SOL_IX_DISCM: u8 = 0;

pub const LST_TO_SOL_IX_DATA_LEN: usize = IX_DATA_LEN;

pub type LstToSolIxData = IxData<LST_TO_SOL_IX_DISCM>;
