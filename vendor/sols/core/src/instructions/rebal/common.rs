use crate::{
    instructions::common::{HoldingProgAccs, PoolHolding},
    typedefs::TokenSolEmpty,
};

pub type RebalPoolHolding<T> = TokenSolEmpty<PoolHolding<T>>;

pub type RebalHoldingProgAccs<T> = TokenSolEmpty<HoldingProgAccs<T>>;

// Use [T; 1] instead of T for as_ref() impl
pub type RebalSvcProgAcc<T> = TokenSolEmpty<[T; 1]>;
