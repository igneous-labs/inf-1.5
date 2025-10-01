use bs58_fixed_wasm::Bs58Array;
use inf1_std::inf1_pp_core::pair::Pair;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    err::InfError,
    interface::{AccountMap, PkPair, B58PK},
    Inf,
};

/// Returned accounts are deduped
///
/// @throws
#[wasm_bindgen(js_name = accountsToUpdateForRebalance)]
pub fn accounts_to_update_for_rebalance(
    inf: &mut Inf,
    PkPair {
        inp: Bs58Array(inp),
        out: Bs58Array(out),
    }: &PkPair,
) -> Result<Box<[B58PK]>, InfError> {
    let mut res: Vec<_> = inf
        .0
        .accounts_to_update_rebalance_mut(&Pair { inp, out })?
        .map(B58PK::new)
        .collect();
    res.sort();
    res.dedup();
    Ok(res.into_boxed_slice())
}

/// @throws
#[wasm_bindgen(js_name = updateForRebalance)]
pub fn update_for_rebalance(
    inf: &mut Inf,
    PkPair {
        inp: Bs58Array(inp),
        out: Bs58Array(out),
    }: &PkPair,
    account_map: &AccountMap,
) -> Result<(), InfError> {
    inf.0.update_rebalance(&Pair { inp, out }, account_map)?;
    Ok(())
}
