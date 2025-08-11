use bs58_fixed_wasm::Bs58Array;
use inf1_std::pda::{
    find_pool_reserves_ata as find_pool_reserves_ata_static,
    find_protocol_fee_accumulator_ata as find_protocol_fee_accumulator_ata_static,
};
use wasm_bindgen::prelude::*;

use crate::{
    err::{no_valid_pda_err, InfError},
    interface::B58PK,
    pda::{find_pda, FoundPda},
};

/// @throws if no valid PDA found
#[wasm_bindgen(js_name = findPoolReservesAta)]
pub fn find_pool_reserves_ata(Bs58Array(mint): &B58PK) -> Result<FoundPda, InfError> {
    find_pool_reserves_ata_static(find_pda, mint)
        .ok_or_else(no_valid_pda_err)
        .map(|(pk, b)| FoundPda(B58PK::new(pk), b))
}

/// @throws if no valid PDA found
#[wasm_bindgen(js_name = findProtocolFeeAccumulatorAta)]
pub fn find_protocol_fee_accumulator_ata(Bs58Array(mint): &B58PK) -> Result<FoundPda, InfError> {
    find_protocol_fee_accumulator_ata_static(find_pda, mint)
        .ok_or_else(no_valid_pda_err)
        .map(|(pk, b)| FoundPda(B58PK::new(pk), b))
}
