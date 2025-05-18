use crate::instructions::{IxArgs, IxData};

use super::{IxPreAccs, IX_PRE_IS_SIGNER, IX_PRE_IS_WRITER};

// Accounts

/// `mint` is that of the input LST to add liquidity of
pub type PriceLpTokensToMintIxPreAccs<T> = IxPreAccs<T>;

pub type PriceLpTokensToMintIxPreKeys<'a> = PriceLpTokensToMintIxPreAccs<&'a [u8; 32]>;

pub type PriceLpTokensToMintIxPreKeysOwned = PriceLpTokensToMintIxPreAccs<[u8; 32]>;

pub type PriceLpTokensToMintIxPreAccFlags = PriceLpTokensToMintIxPreAccs<bool>;

pub const PRICE_LP_TOKENS_TO_MINT_IX_PRE_IS_WRITER: PriceLpTokensToMintIxPreAccFlags =
    IX_PRE_IS_WRITER;

pub const PRICE_LP_TOKENS_TO_MINT_PRE_IS_SIGNER: PriceLpTokensToMintIxPreAccFlags =
    IX_PRE_IS_SIGNER;

// Data

/// amt - amount of input LST to add liquidity of
///
/// sol_value - sol value of `amt` input LST
pub type PriceLpTokensToMintIxArgs = IxArgs;

pub const PRICE_LP_TOKENS_TO_MINT_IX_DISCM: u8 = 2;

pub type PriceLpTokensToMintIxData = IxData<PRICE_LP_TOKENS_TO_MINT_IX_DISCM>;
