use inf1_ctl_core::{
    keys::{
        ATOKEN_ID, DISABLE_POOL_AUTHORITY_LIST_BUMP, LST_STATE_LIST_BUMP, POOL_STATE_BUMP,
        PROTOCOL_FEE_BUMP,
    },
    pda::{
        pool_reserves_ata_seeds, protocol_fee_accumulator_ata_seeds,
        DISABLE_POOL_AUTHORITY_LIST_SEED, LST_STATE_LIST_SEED, POOL_STATE_SEED, PROTOCOL_FEE_SEED,
    },
};
use jiminy_pda::{
    create_raw_program_address, try_find_program_address, PdaSeed, PdaSigner, PDA_MARKER,
};

macro_rules! const_1seed_signer {
    ($NAME:ident, $seed:expr, $bump:expr) => {
        pub const $NAME: PdaSigner =
            PdaSigner::new(&[PdaSeed::new($seed.as_slice()), PdaSeed::new(&[$bump])]);
    };
}

const_1seed_signer!(POOL_STATE_SIGNER, POOL_STATE_SEED, POOL_STATE_BUMP);

const_1seed_signer!(
    LST_STATE_LIST_SIGNER,
    LST_STATE_LIST_SEED,
    LST_STATE_LIST_BUMP
);

const_1seed_signer!(PROTOCOL_FEE_SIGNER, PROTOCOL_FEE_SEED, PROTOCOL_FEE_BUMP);

const_1seed_signer!(
    DISABLE_POOL_AUTH_LIST_SIGNER,
    DISABLE_POOL_AUTHORITY_LIST_SEED,
    DISABLE_POOL_AUTHORITY_LIST_BUMP
);

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
