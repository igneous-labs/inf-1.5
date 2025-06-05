use std::collections::HashMap;

use bs58_fixed_wasm::Bs58Array;
use serde::{Deserialize, Serialize};
use tsify_next::{declare, Tsify};

#[declare]
pub type B58PK = Bs58Array<32, 44>;

#[derive(Debug, Default, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi, hashmap_as_object)]
#[serde(rename_all = "camelCase")]
pub struct AccountMap(pub HashMap<B58PK, Account>);

/// Map of `mint: stake pool account` for spl (all deploys) LSTs.
///
/// This data is required to determine how to properly initialize the corresponding
/// sol value calculator data since which stake pool account corresponds to which mint
/// is not available onchain (yet)
#[derive(Debug, Default, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi, hashmap_as_object)]
#[serde(rename_all = "camelCase")]
pub struct SplPoolAccounts(pub HashMap<B58PK, B58PK>);

#[derive(Debug, Default, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi, large_number_types_as_bigints)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    #[tsify(type = "Uint8Array")] // Instead of number[]
    pub data: Box<[u8]>,

    pub owner: B58PK,
}
