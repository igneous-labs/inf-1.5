use inf1_ctl_core::{
    keys::ATOKEN_ID,
    pda::{pool_reserves_ata_seeds, protocol_fee_accumulator_ata_seeds},
};
use solana_pubkey::Pubkey;

pub fn find_pool_reserves_ata(token_program: &[u8; 32], mint: &[u8; 32]) -> (Pubkey, u8) {
    let [s0, s1, s2] = pool_reserves_ata_seeds(token_program, mint);
    Pubkey::find_program_address(&[s0, s1, s2], &Pubkey::new_from_array(ATOKEN_ID))
}

pub fn find_protocol_fee_accumulator_ata(
    token_program: &[u8; 32],
    mint: &[u8; 32],
) -> (Pubkey, u8) {
    let [s0, s1, s2] = protocol_fee_accumulator_ata_seeds(token_program, mint);
    Pubkey::find_program_address(&[s0, s1, s2], &Pubkey::new_from_array(ATOKEN_ID))
}

pub fn create_pool_reserves_ata(token_program: &[u8; 32], mint: &[u8; 32], bump: u8) -> Pubkey {
    let [s0, s1, s2] = pool_reserves_ata_seeds(token_program, mint);
    Pubkey::create_program_address(&[s0, s1, s2, &[bump]], &Pubkey::new_from_array(ATOKEN_ID))
        .unwrap()
}

pub fn create_protocol_fee_accumulator_ata(
    token_program: &[u8; 32],
    mint: &[u8; 32],
    bump: u8,
) -> Pubkey {
    let [s0, s1, s2] = protocol_fee_accumulator_ata_seeds(token_program, mint);
    Pubkey::create_program_address(&[s0, s1, s2, &[bump]], &Pubkey::new_from_array(ATOKEN_ID))
        .unwrap()
}
