use super::{IxArgs, IxData, IxPreAccs, IX_PRE_IS_SIGNER, IX_PRE_IS_WRITER};

// Accounts

pub type SwapExactInIxPreAccs<T> = IxPreAccs<T>;

pub type SwapExactInIxPreKeys<'a> = SwapExactInIxPreAccs<&'a [u8; 32]>;

pub type SwapExactInIxPreKeysOwned = SwapExactInIxPreAccs<[u8; 32]>;

pub type SwapExactInIxPreAccFlags = SwapExactInIxPreAccs<bool>;

pub const SWAP_EXACT_IN_IX_PRE_IS_WRITER: SwapExactInIxPreAccFlags = IX_PRE_IS_WRITER;

pub const SWAP_EXACT_IN_IX_PRE_IS_SIGNER: SwapExactInIxPreAccFlags = IX_PRE_IS_SIGNER;

// Data

pub const SWAP_EXACT_IN_IX_DISCM: u8 = 1;

/// - limit: min_amount_out
pub type SwapExactInIxArgs = IxArgs;

pub type SwapExactInIxData = IxData<SWAP_EXACT_IN_IX_DISCM>;
