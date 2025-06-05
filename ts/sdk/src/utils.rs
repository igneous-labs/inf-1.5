use inf1_core::inf1_ctl_core::{
    pda::{pool_reserves_ata_seeds, protocol_fee_accumulator_ata_seeds},
    typedefs::lst_state::{LstState, LstStatePacked},
};
use inf1_svc_ag::inf1_svc_marinade_core::sanctum_marinade_liquid_staking_core::{
    ASSOCIATED_TOKEN_PROGRAM, TOKEN_PROGRAM,
};
use wasm_bindgen::JsError;

use crate::{err::unsupported_mint, pda::create_raw_pda};

pub(crate) fn epoch_from_clock_data(clock_acc_data: &[u8]) -> Option<u64> {
    u64_le_at(clock_acc_data, 16)
}

pub(crate) fn token_supply_from_mint_data(mint_acc_data: &[u8]) -> Option<u64> {
    u64_le_at(mint_acc_data, 36)
}

pub(crate) fn balance_from_token_acc_data(token_acc_data: &[u8]) -> Option<u64> {
    u64_le_at(token_acc_data, 64)
}

fn u64_le_at(data: &[u8], at: usize) -> Option<u64> {
    chunk_at(data, at).map(|c| u64::from_le_bytes(*c))
}

fn chunk_at<const N: usize>(data: &[u8], at: usize) -> Option<&[u8; N]> {
    data.get(at..).and_then(|s| s.first_chunk())
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

pub(crate) fn try_find_lst_state(
    packed: &[LstStatePacked],
    mint: &[u8; 32],
) -> Result<(usize, LstState), JsError> {
    packed
        .iter()
        .enumerate()
        .map(|(i, l)| (i, l.into_lst_state()))
        .find(|(_i, l)| l.mint == *mint)
        .ok_or_else(|| unsupported_mint(mint))
}
