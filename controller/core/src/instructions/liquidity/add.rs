use super::{
    IxArgs, IxPreAccs, LiquidityIxData, NewIxPreAccsBuilder, IX_PRE_IS_SIGNER, IX_PRE_IS_WRITER,
};

// Accounts

pub type NewAddLiquidityIxPreAccsBuilder<T> = NewIxPreAccsBuilder<T>;

pub type AddLiquidityIxPreAccs<T> = IxPreAccs<T>;

pub type AddLiquidityIxPreKeys<'a> = AddLiquidityIxPreAccs<&'a [u8; 32]>;

pub type AddLiquidityIxPreKeysOwned = AddLiquidityIxPreAccs<[u8; 32]>;

pub type AddLiquidityIxPreAccFlags = AddLiquidityIxPreAccs<bool>;

pub const ADD_LIQUIDITY_IX_PRE_IS_WRITER: AddLiquidityIxPreAccFlags = IX_PRE_IS_WRITER;

pub const ADD_LIQUIDITY_IX_PRE_IS_SIGNER: AddLiquidityIxPreAccFlags = IX_PRE_IS_SIGNER;

// Data

pub const ADD_LIQUIDITY_IX_DISCM: u8 = 3;

/// - amount: amount of LST tokens to add liquidity of
/// - min_out: min expected amount of LP tokens minted
pub type AddLiquidityIxArgs = IxArgs;

pub type AddLiquidityIxData = LiquidityIxData<ADD_LIQUIDITY_IX_DISCM>;
