//! Special handling of SPL stake pools

use bs58_fixed_wasm::Bs58Array;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    interface::{SplPoolAccounts, B58PK},
    Inf,
};

/// Add SPL LSTs auxiliary data to support new SPL LSTs that may have previously not been covered
#[wasm_bindgen(js_name = appendSplLsts)]
pub fn append_spl_lsts(inf: &mut Inf, SplPoolAccounts(spl_lsts): SplPoolAccounts) {
    inf.0.spl_lsts.extend(
        spl_lsts
            .into_iter()
            .map(|(Bs58Array(k), Bs58Array(v))| (k, v)),
    );
}

/// Returns if the given SPL LST mints have their {@link SplPoolAccounts} present in the object.
///
/// Returns a byte array where ret[i] corresponds to the result for `mints[i]`.
/// 0 - false, 1 - true.
///
/// If false is returned, then the data needs to be added via {@link appendSplLsts}
///
/// This fn returns a byte array instead of `boolean` array because wasm_bindgen's type
/// conversion doesnt work with bool arrays.
#[wasm_bindgen(js_name = hasSplData)]
pub fn has_spl_data(
    inf: &Inf,
    // Clippy complains, needed for wasm_bindgen
    #[allow(clippy::boxed_local)] mints: Box<[B58PK]>,
) -> Box<[u8]> {
    mints
        .iter()
        .map(|mint| u8::from(inf.0.spl_lsts.contains_key(&mint.0)))
        .collect()
}
