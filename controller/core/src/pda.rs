use const_crypto::ed25519::derive_program_address;

use crate::keys::{ATOKEN_ID, POOL_STATE_ID, PROTOCOL_FEE_ID};

pub const POOL_STATE_SEED: [u8; 5] = *b"state";

pub const LST_STATE_LIST_SEED: [u8; 14] = *b"lst-state-list";

pub const PROTOCOL_FEE_SEED: [u8; 12] = *b"protocol-fee";

pub const REBALANCE_RECORD_SEED: [u8; 16] = *b"rebalance-record";

// TOOD: disable-pool-authority-list

pub const fn const_find_pool_state(prog_id: &[u8; 32]) -> ([u8; 32], u8) {
    derive_program_address(&[&POOL_STATE_SEED], prog_id)
}

pub const fn const_find_lst_state_list(prog_id: &[u8; 32]) -> ([u8; 32], u8) {
    derive_program_address(&[&LST_STATE_LIST_SEED], prog_id)
}

pub const fn const_find_protocol_fee(prog_id: &[u8; 32]) -> ([u8; 32], u8) {
    derive_program_address(&[&PROTOCOL_FEE_SEED], prog_id)
}

pub const fn const_find_rebalance_record(prog_id: &[u8; 32]) -> ([u8; 32], u8) {
    derive_program_address(&[&REBALANCE_RECORD_SEED], prog_id)
}

pub const fn const_find_pool_reserves(token_program: &[u8; 32], mint: &[u8; 32]) -> ([u8; 32], u8) {
    let [s1, s2, s3] = pool_reserves_ata_seeds(token_program, mint);
    derive_program_address(&[s1, s2, s3], &ATOKEN_ID)
}

pub const fn const_find_protocol_fee_accumulator(
    token_program: &[u8; 32],
    mint: &[u8; 32],
) -> ([u8; 32], u8) {
    let [s1, s2, s3] = protocol_fee_accumulator_ata_seeds(token_program, mint);
    derive_program_address(&[s1, s2, s3], &ATOKEN_ID)
}

/// PDA seeds to use with ATA program to find pool reserves ATA
pub const fn pool_reserves_ata_seeds<'a>(
    token_program: &'a [u8; 32],
    mint: &'a [u8; 32],
) -> [&'a [u8; 32]; 3] {
    [&POOL_STATE_ID, token_program, mint]
}

/// PDA seeds to use with ATA program to find protocol fee accumulator ATA
pub const fn protocol_fee_accumulator_ata_seeds<'a>(
    token_program: &'a [u8; 32],
    mint: &'a [u8; 32],
) -> [&'a [u8; 32]; 3] {
    [&PROTOCOL_FEE_ID, token_program, mint]
}
