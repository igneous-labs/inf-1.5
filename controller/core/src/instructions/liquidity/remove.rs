use super::{IxArgs, IxData, IxPreAccs, IX_PRE_IS_SIGNER, IX_PRE_IS_WRITER};

// Accounts

pub type RemoveLiquidityIxPreAccs<T> = IxPreAccs<T>;

pub type RemoveLiquidityIxPreKeys<'a> = RemoveLiquidityIxPreAccs<&'a [u8; 32]>;

pub type RemoveLiquidityIxPreKeysOwned = RemoveLiquidityIxPreAccs<[u8; 32]>;

pub type RemoveLiquidityIxPreAccFlags = RemoveLiquidityIxPreAccs<bool>;

pub const REMOVE_LIQUIDITY_IX_PRE_IS_WRITER: RemoveLiquidityIxPreAccFlags = IX_PRE_IS_WRITER;

pub const REMOVE_LIQUIDITY_IX_PRE_IS_SIGNER: RemoveLiquidityIxPreAccFlags = IX_PRE_IS_SIGNER;

// Data

pub const REMOVE_LIQUIDITY_IX_DISCM: u8 = 4;

/// - amount: amount of LP tokens to redeem
/// - min_out: min expected amount of LST tokens returned
pub type RemoveLiquidityIxArgs = IxArgs;

pub type RemoveLiquidityIxData = IxData<REMOVE_LIQUIDITY_IX_DISCM>;
