use solana_pubkey::Pubkey;

pub fn find_pda(seeds: &[&[u8]], prog_id: &[u8; 32]) -> Option<([u8; 32], u8)> {
    Pubkey::try_find_program_address(seeds, &Pubkey::new_from_array(*prog_id))
        .map(|(a, b)| (a.to_bytes(), b))
}

pub fn create_pda(seeds: &[&[u8]], prog_id: &[u8; 32]) -> Option<[u8; 32]> {
    Pubkey::create_program_address(seeds, &Pubkey::new_from_array(*prog_id))
        .ok()
        .map(|a| a.to_bytes())
}
