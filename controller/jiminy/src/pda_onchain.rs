use inf1_ctl_core::{
    keys::{ATOKEN_ID, DISABLE_POOL_AUTHORITY_LIST_BUMP},
    pda::{
        pool_reserves_ata_seeds, protocol_fee_accumulator_ata_seeds,
        DISABLE_POOL_AUTHORITY_LIST_SEED,
    },
};
use jiminy_pda::{
    create_raw_program_address, try_find_program_address, PdaSeed, PdaSigner, PDA_MARKER,
};

pub const DISABLE_POOL_AUTH_LIST_SIGNER: PdaSigner = PdaSigner::new(&[
    PdaSeed::new(DISABLE_POOL_AUTHORITY_LIST_SEED.as_slice()),
    PdaSeed::new(&[DISABLE_POOL_AUTHORITY_LIST_BUMP]),
]);

#[inline]
pub fn create_raw_pool_reserves_addr(
    token_program: &[u8; 32],
    mint: &[u8; 32],
    bump: &u8,
) -> Option<[u8; 32]> {
    let [s0, s1, s2] =
        pool_reserves_ata_seeds(token_program, mint).map(|s| PdaSeed::new(s.as_slice()));
    let seeds = [
        s0,
        s1,
        s2,
        PdaSeed::new(core::slice::from_ref(bump)),
        PdaSeed::new(&ATOKEN_ID),
        PdaSeed::new(&PDA_MARKER),
    ];
    create_raw_program_address(&seeds)
}

#[inline]
pub fn create_raw_protocol_fee_accumulator_addr(
    token_program: &[u8; 32],
    mint: &[u8; 32],
    bump: &u8,
) -> Option<[u8; 32]> {
    let [s0, s1, s2] =
        protocol_fee_accumulator_ata_seeds(token_program, mint).map(|s| PdaSeed::new(s.as_slice()));
    let seeds = [
        s0,
        s1,
        s2,
        PdaSeed::new(core::slice::from_ref(bump)),
        PdaSeed::new(&ATOKEN_ID),
        PdaSeed::new(&PDA_MARKER),
    ];
    create_raw_program_address(&seeds)
}

#[inline]
pub fn find_pool_reserves(token_program: &[u8; 32], mint: &[u8; 32]) -> Option<([u8; 32], u8)> {
    let [s0, s1, s2] =
        pool_reserves_ata_seeds(token_program, mint).map(|s| PdaSeed::new(s.as_slice()));
    try_find_program_address(&[s0, s1, s2], &ATOKEN_ID)
}

#[inline]
pub fn find_protocol_fee_accumulator(
    token_program: &[u8; 32],
    mint: &[u8; 32],
) -> Option<([u8; 32], u8)> {
    let [s0, s1, s2] =
        protocol_fee_accumulator_ata_seeds(token_program, mint).map(|s| PdaSeed::new(s.as_slice()));
    try_find_program_address(&[s0, s1, s2], &ATOKEN_ID)
}
