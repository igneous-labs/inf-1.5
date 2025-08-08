use bs58_fixed_wasm::Bs58Array;

use wasm_bindgen::prelude::*;

use crate::{
    err::InfError,
    interface::{AccountMap, PkPair, B58PK},
    trade::Pair,
    Inf,
};

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
    let lp_token_mint = inf.0.pool().lp_token_mint;

    let mut res: Vec<_> = if *out == lp_token_mint {
        // add liquidity
        inf.0
            .accounts_to_update_add_liq_mut(inp)?
            .map(B58PK::new)
            .collect()
    } else if *inp == lp_token_mint {
        // remove liquidity
        inf.0
            .accounts_to_update_remove_liq_mut(out)?
            .map(B58PK::new)
            .collect()
    } else {
        // swap
        // TODO: currently this assumes no difference in accounts between
        // SwapExactIn and SwapExactOut. This might change in the future
        inf.0
            .accounts_to_update_swap_exact_in_mut(&Pair { inp, out })?
            .map(B58PK::new)
            .collect()
    };

    res.sort();
    res.dedup();
    Ok(res.into_boxed_slice())
}

// TODO: this currently assumes accounts required for ExactIn
// and ExactOut are the same, tho this might not be the case for future pricing programs
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
    let lp_token_mint = inf.0.pool().lp_token_mint;
    if *out == lp_token_mint {
        // add liquidity
        inf.0.update_add_liq(inp, account_map)?;
    } else if *inp == lp_token_mint {
        // remove liquidity
        inf.0.update_remove_liq(out, account_map)?;
    } else {
        // swap
        // TODO: currently this assumes no difference in accounts between
        // SwapExactIn and SwapExactOut. This might change in the future
        inf.0
            .update_swap_exact_in(&Pair { inp, out }, account_map)?;
    };

    Ok(())
}
