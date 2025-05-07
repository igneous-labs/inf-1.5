pub const STATE_SEED: [u8; 5] = *b"state";

pub const fn const_find_state(prog_id: &[u8; 32]) -> ([u8; 32], u8) {
    const_crypto::ed25519::derive_program_address(&[&STATE_SEED], prog_id)
}
