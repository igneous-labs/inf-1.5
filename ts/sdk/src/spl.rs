//! Special handling of SPL stake pools

use bs58_fixed_wasm::Bs58Array;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{interface::SplPoolAccounts, Inf};

/// Update SPL LSTs auxiliary data to support new SPL LSTs that may have previously not been covered
#[wasm_bindgen(js_name = updateSplLsts)]
pub fn update_spl_lsts(inf: &mut Inf, SplPoolAccounts(spl_lsts): SplPoolAccounts) {
    inf.spl_lsts = spl_lsts
        .into_iter()
        .map(|(Bs58Array(k), Bs58Array(v))| (k, v))
        .collect();
}
