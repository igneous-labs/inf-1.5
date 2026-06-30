use const_crypto::ed25519::derive_program_address;

use crate::{
    keys::{POOL_STATE_ID, PROTOCOL_FEE_ID},
    token_info::TokenInfo,
};

pub const POOL_STATE_SEED: [u8; 5] = *b"state";

pub const LST_STATE_LIST_SEED: [u8; 14] = *b"lst-state-list";

pub const PROTOCOL_FEE_SEED: [u8; 12] = *b"protocol-fee";

pub const REBALANCE_RECORD_SEED: [u8; 16] = *b"rebalance-record";

pub const DISABLE_POOL_AUTHORITY_LIST_SEED: [u8; 27] = *b"disable-pool-authority-list";

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

pub const fn const_find_disable_pool_authority_list(prog_id: &[u8; 32]) -> ([u8; 32], u8) {
    derive_program_address(&[&DISABLE_POOL_AUTHORITY_LIST_SEED], prog_id)
}

/// PDA seeds to use with ATA program to find pool reserves ATA
pub const fn pool_reserves_ata_seeds<'a>(token: &TokenInfo<&'a [u8; 32]>) -> [&'a [u8; 32]; 3] {
    ata_seeds(&POOL_STATE_ID, token)
}

/// PDA seeds to use with ATA program to find protocol fee accumulator ATA
pub const fn protocol_fee_accumulator_ata_seeds<'a>(
    token: &TokenInfo<&'a [u8; 32]>,
) -> [&'a [u8; 32]; 3] {
    ata_seeds(&PROTOCOL_FEE_ID, token)
}

/// PDA seeds to use with ATA program to find ATA addr
pub const fn ata_seeds<'a>(
    auth: &'a [u8; 32],
    TokenInfo([program, mint]): &TokenInfo<&'a [u8; 32]>,
) -> [&'a [u8; 32]; 3] {
    [auth, program, mint]
}
