use crate::instructions::{IxArgs, IxData};

use super::{IxPreAccs, IX_PRE_IS_SIGNER, IX_PRE_IS_WRITER};

// Accounts

pub type PriceExactOutIxPreAccs<T> = IxPreAccs<T>;

pub type PriceExactOutIxPreKeys<'a> = PriceExactOutIxPreAccs<&'a [u8; 32]>;

pub type PriceExactOutIxPreKeysOwned = PriceExactOutIxPreAccs<[u8; 32]>;

pub type PriceExactOutIxPreAccFlags = PriceExactOutIxPreAccs<bool>;

pub const PRICE_EXACT_OUT_IX_PRE_IS_WRITER: PriceExactOutIxPreAccFlags = IX_PRE_IS_WRITER;

pub const PRICE_EXACT_OUT_PRE_IS_SIGNER: PriceExactOutIxPreAccFlags = IX_PRE_IS_SIGNER;

// Data

/// amt - amount of output LST
///
/// sol_value - sol value of `amt` output LST
pub type PriceExactOutIxArgs = IxArgs;

pub const PRICE_EXACT_OUT_IX_DISCM: u8 = 1;

pub type PriceExactOutIxData = IxData<PRICE_EXACT_OUT_IX_DISCM>;
