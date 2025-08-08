use bs58_fixed_wasm::Bs58Array;
use inf1_std::inf1_ctl_core::pda::{pool_reserves_ata_seeds, protocol_fee_accumulator_ata_seeds};
use inf1_svc_ag_core::inf1_svc_marinade_core::sanctum_marinade_liquid_staking_core::{
    ASSOCIATED_TOKEN_PROGRAM, TOKEN_PROGRAM,
};
use wasm_bindgen::prelude::*;

use crate::{
    err::{no_valid_pda_err, InfError},
    interface::B58PK,
    pda::{create_raw_pda, find_pda, FoundPda},
};

/// @throws if not valid PDA found
#[wasm_bindgen(js_name = findPoolReservesAta)]
pub fn find_pool_reserves_ata(Bs58Array(mint): &B58PK) -> Result<FoundPda, InfError> {
    let [s1, s2, s3] = pool_reserves_ata_seeds(&TOKEN_PROGRAM, mint);
    find_pda(&[s1, s2, s3], &ASSOCIATED_TOKEN_PROGRAM)
        .ok_or_else(no_valid_pda_err)
        .map(|(pk, b)| FoundPda(B58PK::new(pk), b))
}

/// @throws if not valid PDA found
#[wasm_bindgen(js_name = findProtocolFeeAccumulatorAta)]
pub fn find_protocol_fee_accumulator_ata(Bs58Array(mint): &B58PK) -> Result<FoundPda, InfError> {
    let [s1, s2, s3] = protocol_fee_accumulator_ata_seeds(&TOKEN_PROGRAM, mint);
    find_pda(&[s1, s2, s3], &ASSOCIATED_TOKEN_PROGRAM)
        .ok_or_else(no_valid_pda_err)
        .map(|(pk, b)| FoundPda(B58PK::new(pk), b))
}

// TODO: need to create variant that supports token-22 if needed
pub(crate) fn create_raw_pool_reserves_ata(mint: &[u8; 32], bump: u8) -> [u8; 32] {
    let [s1, s2, s3] = pool_reserves_ata_seeds(&TOKEN_PROGRAM, mint);
    // unwrap-safety: seeds are within range
    create_raw_pda([s1.as_slice(), s2, s3, &[bump]], &ASSOCIATED_TOKEN_PROGRAM).unwrap()
}

// TODO: need to create variant that supports token-22 if needed
pub(crate) fn create_raw_protocol_fee_accumulator_ata(mint: &[u8; 32], bump: u8) -> [u8; 32] {
    let [s1, s2, s3] = protocol_fee_accumulator_ata_seeds(&TOKEN_PROGRAM, mint);
    // unwrap-safety: seeds are within range
    create_raw_pda([s1.as_slice(), s2, s3, &[bump]], &ASSOCIATED_TOKEN_PROGRAM).unwrap()
}
