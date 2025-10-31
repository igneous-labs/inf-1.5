use inf1_ctl_core::{
    keys::ATOKEN_ID,
    pda::{pool_reserves_ata_seeds, protocol_fee_accumulator_ata_seeds},
};
use jiminy_pda::{create_raw_program_address, PdaSeed, PDA_MARKER};

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
