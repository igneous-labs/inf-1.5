//! The main quoting + swapping functionality

use serde::{Deserialize, Serialize};
use tsify_next::Tsify;
use wasm_bindgen::prelude::*;

use crate::interface::B58PK;

mod instruction;
mod quote;
mod update;

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, Tsify, PartialEq, Eq, Hash)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct Pair<T> {
    pub inp: T,
    pub out: T,
}

// need to use a simple newtype here instead of type alias
// otherwise wasm_bindgen shits itself with missing generics
#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, Tsify, PartialEq, Eq, Hash)]
#[tsify(into_wasm_abi, from_wasm_abi, large_number_types_as_bigints)]
#[serde(rename_all = "camelCase")]
pub struct PkPair(pub(crate) Pair<B58PK>);
