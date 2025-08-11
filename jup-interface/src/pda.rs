use solana_pubkey::Pubkey;
use solana_sha256_hasher::Hasher;

const PDA_MARKER: &[u8; 21] = b"ProgramDerivedAddress";

/// This fn omits the following checks for performance, at the cost of safety:
/// - does not check if seed lenghts are within bounds
/// - does not check if resulting PDA is indeed not on curve
///
/// The args to this fn must be guaranteed to be of a valid PDA
pub(crate) fn create_raw_pda(seeds: &[&[u8]], program_id: &[u8; 32]) -> Option<[u8; 32]> {
    let mut hasher = Hasher::default();
    for seed in seeds.iter() {
        hasher.hash(seed);
    }
    hasher.hashv(&[program_id.as_ref(), PDA_MARKER]);
    let hash = hasher.result();

    Some(hash.to_bytes())
}

pub(crate) fn find_pda(seeds: &[&[u8]], program_id: &[u8; 32]) -> Option<([u8; 32], u8)> {
    Pubkey::try_find_program_address(seeds, &Pubkey::new_from_array(*program_id))
        .map(|(pk, bump)| (pk.to_bytes(), bump))
}
