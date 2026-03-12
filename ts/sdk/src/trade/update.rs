use bs58_fixed_wasm::Bs58Array;

use inf1_std::trade::TradeLimitTy;
use wasm_bindgen::prelude::*;

use crate::{
    err::InfError,
    interface::{AccountMap, PkPair, B58PK},
    trade::Pair,
    Inf,
};

// TODO: these update procedures currently assumes accounts required for ExactIn
// and ExactOut are the same, tho this might not be the case for future pricing programs

/// Returned accounts are deduped
///
/// @throws
#[wasm_bindgen(js_name = accountsToUpdateForTrade)]
pub fn accounts_to_update_for_trade(
    inf: &mut Inf,
    PkPair {
        inp: Bs58Array(inp),
        out: Bs58Array(out),
    }: &PkPair,
) -> Result<Box<[B58PK]>, InfError> {
    let mut res: Vec<_> = inf
        .0
        .accounts_to_update_trade_mut(&Pair { inp, out }, TradeLimitTy::ExactIn(()))?
        .map(B58PK::new)
        .collect();
    res.sort();
    res.dedup();
    Ok(res.into_boxed_slice())
}

/// @throws
#[wasm_bindgen(js_name = updateForTrade)]
pub fn update_for_trade(
    inf: &mut Inf,
    PkPair {
        inp: Bs58Array(inp),
        out: Bs58Array(out),
    }: &PkPair,
    account_map: &AccountMap,
) -> Result<(), InfError> {
    inf.0
        .update_trade(&Pair { inp, out }, TradeLimitTy::ExactIn(()), account_map)?;
    Ok(())
}
