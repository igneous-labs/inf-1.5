use inf1_ctl_jiminy::{keys::ATOKEN_ID, pda::pool_reserves_ata_seeds};
use solana_pubkey::Pubkey;

pub fn find_pool_reserves(token_program: &[u8; 32], mint: &[u8; 32]) -> (Pubkey, u8) {
    let [s0, s1, s2] = pool_reserves_ata_seeds(token_program, mint);
    Pubkey::find_program_address(&[s0, s1, s2], &Pubkey::new_from_array(ATOKEN_ID))
}
