use crate::instructions::{IxArgs, IxData};

use super::{IxPreAccs, IX_PRE_IS_SIGNER, IX_PRE_IS_WRITER};

// Accounts

pub type PriceExactInIxPreAccs<T> = IxPreAccs<T>;

pub type PriceExactInIxPreKeys<'a> = PriceExactInIxPreAccs<&'a [u8; 32]>;

pub type PriceExactInIxPreKeysOwned = PriceExactInIxPreAccs<[u8; 32]>;

pub type PriceExactInIxPreAccFlags = PriceExactInIxPreAccs<bool>;

pub const PRICE_EXACT_IN_IX_PRE_IS_WRITER: PriceExactInIxPreAccFlags = IX_PRE_IS_WRITER;

pub const PRICE_EXACT_IN_PRE_IS_SIGNER: PriceExactInIxPreAccFlags = IX_PRE_IS_SIGNER;

// Data

/// amt - amount of input LST
///
/// sol_value - sol value of `amt` input LST
pub type PriceExactInIxArgs = IxArgs;

pub const PRICE_EXACT_IN_IX_DISCM: u8 = 0;

pub type PriceExactInIxData = IxData<PRICE_EXACT_IN_IX_DISCM>;
