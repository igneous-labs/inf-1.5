use crate::instructions::swap::{IxArgs, IxData};

use super::{IxPreAccs, NewIxPreAccsBuilder, IX_PRE_IS_SIGNER, IX_PRE_IS_WRITER};

// Accounts

pub type NewSwapExactOutIxPreAccsBuilder<T> = NewIxPreAccsBuilder<T>;

pub type SwapExactOutIxPreAccs<T> = IxPreAccs<T>;

pub type SwapExactOutIxPreKeys<'a> = SwapExactOutIxPreAccs<&'a [u8; 32]>;

pub type SwapExactOutIxPreKeysOwned = SwapExactOutIxPreAccs<[u8; 32]>;

pub type SwapExactOutIxPreAccFlags = SwapExactOutIxPreAccs<bool>;

pub const SWAP_EXACT_OUT_IX_PRE_IS_WRITER: SwapExactOutIxPreAccFlags = IX_PRE_IS_WRITER;

pub const SWAP_EXACT_OUT_IX_PRE_IS_SIGNER: SwapExactOutIxPreAccFlags = IX_PRE_IS_SIGNER;

// Data

pub const SWAP_EXACT_OUT_IX_DISCM: u8 = 2;

/// - limit: min_amount_out
pub type SwapExactOutIxArgs = IxArgs;

pub type SwapExactOutIxData = IxData<SWAP_EXACT_OUT_IX_DISCM>;
