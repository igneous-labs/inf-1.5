use crate::instructions::swap::{IxArgs, IxData};

use super::{IxPreAccs, NewIxPreAccsBuilder, IX_PRE_IS_SIGNER, IX_PRE_IS_WRITER};

// Accounts

pub type NewSwapExactOutV2IxPreAccsBuilder<T> = NewIxPreAccsBuilder<T>;

pub type SwapExactOutV2IxPreAccs<T> = IxPreAccs<T>;

pub type SwapExactOutV2IxPreKeys<'a> = SwapExactOutV2IxPreAccs<&'a [u8; 32]>;

pub type SwapExactOutV2IxPreKeysOwned = SwapExactOutV2IxPreAccs<[u8; 32]>;

pub type SwapExactOutV2IxPreAccFlags = SwapExactOutV2IxPreAccs<bool>;

pub const SWAP_EXACT_OUT_V2_IX_PRE_IS_WRITER: SwapExactOutV2IxPreAccFlags = IX_PRE_IS_WRITER;

pub const SWAP_EXACT_OUT_V2_IX_PRE_IS_SIGNER: SwapExactOutV2IxPreAccFlags = IX_PRE_IS_SIGNER;

// Data

pub const SWAP_EXACT_OUT_V2_IX_DISCM: u8 = 24;

/// - limit: max_amount_in
pub type SwapExactOutIxArgs = IxArgs;

pub type SwapExactOutIxData = IxData<SWAP_EXACT_OUT_V2_IX_DISCM>;
