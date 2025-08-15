use crate::instructions::{IxArgs, IxData};

use super::{IxPreAccs, IX_PRE_IS_SIGNER, IX_PRE_IS_WRITER};

// Accounts

/// `mint` is that of the output LST to remove liquidity of
pub type PriceLpTokensToRedeemIxPreAccs<T> = IxPreAccs<T>;

pub type PriceLpTokensToRedeemIxPreKeys<'a> = PriceLpTokensToRedeemIxPreAccs<&'a [u8; 32]>;

pub type PriceLpTokensToRedeemIxPreKeysOwned = PriceLpTokensToRedeemIxPreAccs<[u8; 32]>;

pub type PriceLpTokensToRedeemIxPreAccFlags = PriceLpTokensToRedeemIxPreAccs<bool>;

pub const PRICE_LP_TOKENS_TO_REDEEM_IX_PRE_IS_WRITER: PriceLpTokensToRedeemIxPreAccFlags =
    IX_PRE_IS_WRITER;

pub const PRICE_LP_TOKENS_TO_REDEEM_IX_PRE_IS_SIGNER: PriceLpTokensToRedeemIxPreAccFlags =
    IX_PRE_IS_SIGNER;

// Data

/// amt - amount of output LST to remove liquidity of
///
/// sol_value - sol value of `amt` output LST
pub type PriceLpTokensToRedeemIxArgs = IxArgs;

pub const PRICE_LP_TOKENS_TO_REDEEM_IX_DISCM: u8 = 3;

pub type PriceLpTokensToRedeemIxData = IxData<PRICE_LP_TOKENS_TO_REDEEM_IX_DISCM>;
