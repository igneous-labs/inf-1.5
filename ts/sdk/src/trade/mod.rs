//! The main quoting + swapping functionality

use serde::{Deserialize, Serialize};
use tsify_next::Tsify;
use wasm_bindgen::prelude::*;

use crate::interface::B58PK;

mod instruction;
mod quote;
mod update;

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct PkPair {
    pub inp: B58PK,
    pub out: B58PK,
}
