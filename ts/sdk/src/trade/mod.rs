//! The main quoting + swapping functionality

use inf1_core::inf1_pp_core::pair::Pair;
use serde::{Deserialize, Serialize};
use tsify_next::Tsify;
use wasm_bindgen::prelude::*;

use crate::interface::B58PK;

mod instruction;
mod quote;
mod update;

// need to use a simple newtype here instead of type alias
// otherwise wasm_bindgen shits itself with missing generics
#[derive(Debug, Default, Clone, Copy, Tsify, PartialEq, Eq, Hash)]
#[tsify(into_wasm_abi, from_wasm_abi, large_number_types_as_bigints)]
#[serde(rename_all = "camelCase")]
pub struct PkPair(pub(crate) Pair<B58PK>);

// use private helper struct + derive(serde) to correctly implement struct/map serialization
// to workaround `inf1_core::inf1_pp_core::pair::Pair` not implementing serde
#[derive(Serialize, Deserialize)]
struct PkPairSerde {
    inp: B58PK,
    out: B58PK,
}

impl Serialize for PkPair {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let Self(Pair { inp, out }) = *self;
        PkPairSerde { inp, out }.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for PkPair {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let PkPairSerde { inp, out } = PkPairSerde::deserialize(deserializer)?;
        Ok(Self(Pair { inp, out }))
    }
}
