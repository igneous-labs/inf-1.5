use crate::instructions::swap::{IxArgs, IxData};

use super::{IxPreAccs, NewIxPreAccsBuilder, IX_PRE_IS_SIGNER, IX_PRE_IS_WRITER};

// Accounts

pub type NewSwapExactInV2IxPreAccsBuilder<T> = NewIxPreAccsBuilder<T>;

pub type SwapExactInV2IxPreAccs<T> = IxPreAccs<T>;

pub type SwapExactInV2IxPreKeys<'a> = SwapExactInV2IxPreAccs<&'a [u8; 32]>;

pub type SwapExactInV2IxPreKeysOwned = SwapExactInV2IxPreAccs<[u8; 32]>;

pub type SwapExactInV2IxPreAccFlags = SwapExactInV2IxPreAccs<bool>;

pub const SWAP_EXACT_IN_V2_IX_PRE_IS_WRITER: SwapExactInV2IxPreAccFlags = IX_PRE_IS_WRITER;

pub const SWAP_EXACT_IN_V2_IX_PRE_IS_SIGNER: SwapExactInV2IxPreAccFlags = IX_PRE_IS_SIGNER;

// Data

pub const SWAP_EXACT_IN_V2_IX_DISCM: u8 = 23;

/// - limit: min_amount_out
pub type SwapExactInIxArgs = IxArgs;

pub type SwapExactInIxData = IxData<SWAP_EXACT_IN_V2_IX_DISCM>;
