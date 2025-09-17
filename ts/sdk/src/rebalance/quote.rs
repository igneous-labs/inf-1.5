use bs58_fixed_wasm::Bs58Array;
use inf1_std::inf1_pp_core::pair::Pair;
use serde::{Deserialize, Serialize};
use tsify_next::Tsify;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{err::InfError, interface::PkPair, Inf};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi, large_number_types_as_bigints)]
#[serde(rename_all = "camelCase")]
pub struct RebalanceQuoteArgs {
    /// Amount of output tokens that will leave the pool in StartRebalance
    pub out: u64,
    pub mints: PkPair,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi, large_number_types_as_bigints)]
#[serde(rename_all = "camelCase")]
pub struct RebalanceQuote {
    /// Amount of input tokens that needs to enter the pool by EndRebalance
    pub inp: u64,

    /// Amount of output tokens that will leave the pool in StartRebalance
    pub out: u64,

    pub mints: PkPair,
}

#[wasm_bindgen(js_name = quoteRebalance)]
pub fn quote_rebalance(
    inf: &mut Inf,
    RebalanceQuoteArgs { out, mints }: &RebalanceQuoteArgs,
) -> Result<RebalanceQuote, InfError> {
    let PkPair {
        inp: Bs58Array(inp_mint),
        out: Bs58Array(out_mint),
    } = mints;
    let inf1_std::quote::rebalance::RebalanceQuote { inp, out, .. } =
        inf.0.quote_rebalance_exact_out_mut(
            &Pair {
                inp: inp_mint,
                out: out_mint,
            },
            *out,
        )?;
    Ok(RebalanceQuote {
        inp,
        out,
        mints: *mints,
    })
}
