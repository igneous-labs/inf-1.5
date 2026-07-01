use inf1_ctl_core::{
    keys::CONST_KEYS_OWNED,
    pda::{
        pool_reserves_ata_seeds, protocol_fee_accumulator_ata_seeds, CONST_PDA_BUMPS,
        DISABLE_POOL_AUTHORITY_LIST_SEED, LST_STATE_LIST_SEED, POOL_STATE_SEED, PROTOCOL_FEE_SEED,
        REBALANCE_RECORD_SEED,
    },
    token_info::TokenInfo,
};
use jiminy_pda::{
    create_raw_program_address, try_find_program_address, PdaSeed, PdaSigner, PDA_MARKER,
};

/// # Note on SBF toolchain issue
///
/// Keeping individual `PdaSigner`s as separate constants via `const_1seed_signer` instead of
/// using a const generic-array-struct aggregate like the other consts because doing that
/// seems to trigger a miscompile bug in the onchain program where
///
/// `invoke_signed(..., &[*CONST_PDA_SIGNERS.pool_state()])`
///
/// works but
///
/// `invoke_signed(..., core::slice::from_ref(CONST_PDA_SIGNERS.pool_state()))`
///
/// fails with `signer privilege escalation`.
///
/// Possibly because the first one copies the `PdaSeed`s data from .rodata onto the stack
/// and then passes pointer to that stack data to invoke_signed while the latter passes a
/// pointer to .rodata directly and the sol_invoke_signed_c syscall has some issues with the latter.
///
/// Or maybe its a lifetime issue with jiminy-cpi
///
/// Whatever the issue, im not dealing with it right now
macro_rules! const_1seed_signer {
    ($NAME:ident, $seed:expr, $bump_ref:expr) => {
        pub const $NAME: PdaSigner = PdaSigner::new(&[
            PdaSeed::new($seed.as_slice()),
            PdaSeed::new(core::slice::from_ref($bump_ref)),
        ]);
    };
}

const_1seed_signer!(
    POOL_STATE_SIGNER,
    POOL_STATE_SEED,
    CONST_PDA_BUMPS.pool_state()
);
const_1seed_signer!(
    LST_STATE_LIST_SIGNER,
    LST_STATE_LIST_SEED,
    CONST_PDA_BUMPS.lst_state_list()
);
const_1seed_signer!(
    PROTOCOL_FEE_SIGNER,
    PROTOCOL_FEE_SEED,
    CONST_PDA_BUMPS.protocol_fee()
);
const_1seed_signer!(
    DISABLE_POOL_AUTHORITY_LIST_SIGNER,
    DISABLE_POOL_AUTHORITY_LIST_SEED,
    CONST_PDA_BUMPS.disable_pool_authority_list()
);
const_1seed_signer!(
    REBALANCE_RECORD_SIGNER,
    REBALANCE_RECORD_SEED,
    CONST_PDA_BUMPS.rebalance_record()
);

#[inline]
pub fn create_raw_pool_reserves_addr(token: &TokenInfo<&[u8; 32]>, bump: &u8) -> Option<[u8; 32]> {
    let [s0, s1, s2] = pool_reserves_ata_seeds(token).map(|s| PdaSeed::new(s.as_slice()));
    let seeds = [
        s0,
        s1,
        s2,
        PdaSeed::new(core::slice::from_ref(bump)),
        PdaSeed::new(CONST_KEYS_OWNED.atoken()),
        PdaSeed::new(&PDA_MARKER),
    ];
    create_raw_program_address(&seeds)
}

#[inline]
pub fn create_raw_protocol_fee_accumulator_addr(
    token: &TokenInfo<&[u8; 32]>,
    bump: &u8,
) -> Option<[u8; 32]> {
    let [s0, s1, s2] =
        protocol_fee_accumulator_ata_seeds(token).map(|s| PdaSeed::new(s.as_slice()));
    let seeds = [
        s0,
        s1,
        s2,
        PdaSeed::new(core::slice::from_ref(bump)),
        PdaSeed::new(CONST_KEYS_OWNED.atoken()),
        PdaSeed::new(&PDA_MARKER),
    ];
    create_raw_program_address(&seeds)
}

#[inline]
pub fn find_pool_reserves(token: &TokenInfo<&[u8; 32]>) -> Option<([u8; 32], u8)> {
    let [s0, s1, s2] = pool_reserves_ata_seeds(token).map(|s| PdaSeed::new(s.as_slice()));
    try_find_program_address(&[s0, s1, s2], CONST_KEYS_OWNED.atoken())
}

#[inline]
pub fn find_protocol_fee_accumulator(token: &TokenInfo<&[u8; 32]>) -> Option<([u8; 32], u8)> {
    let [s0, s1, s2] =
        protocol_fee_accumulator_ata_seeds(token).map(|s| PdaSeed::new(s.as_slice()));
    try_find_program_address(&[s0, s1, s2], CONST_KEYS_OWNED.atoken())
}
