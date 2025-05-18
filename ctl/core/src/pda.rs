use const_crypto::ed25519::derive_program_address;

pub const POOL_STATE_SEED: [u8; 5] = *b"state";

pub const LST_STATE_LIST_SEED: [u8; 14] = *b"lst-state-list";

pub const PROTOCOL_FEE_SEED: [u8; 12] = *b"protocol-fee";

// TOOD: disable-pool-authority-list, rebalance-record

pub const fn const_find_pool_state(prog_id: &[u8; 32]) -> ([u8; 32], u8) {
    derive_program_address(&[&POOL_STATE_SEED], prog_id)
}

pub const fn const_find_lst_state_list(prog_id: &[u8; 32]) -> ([u8; 32], u8) {
    derive_program_address(&[&LST_STATE_LIST_SEED], prog_id)
}

pub const fn const_find_protocol_fee(prog_id: &[u8; 32]) -> ([u8; 32], u8) {
    derive_program_address(&[&PROTOCOL_FEE_SEED], prog_id)
}
