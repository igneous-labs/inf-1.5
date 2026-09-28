use core::{mem::MaybeUninit, slice};

use jiminy_pda::{
    create_raw_program_address_to, try_find_program_address, PdaSeed, PdaSigner, PDA_MARKER,
};
use sanctum_sols_core::{
    err::{InvalidKnownPdaErr, SanctumSolsErr},
    keys::{CONST_KEYS_OWNED, CONST_PDA_BUMPS},
    pda::{portfolio_pda_seeds, protocol_pda_seeds, wsol_bridge_pda_seeds},
};

use crate::onchain::err::SanctumSolsProgErr;

pub const PROTOCOL_PDA_SIGNER: PdaSigner = PdaSigner::new(&[
    PdaSeed::new(protocol_pda_seeds()),
    PdaSeed::new(core::slice::from_ref(CONST_PDA_BUMPS.protocol())),
]);

#[inline]
pub const fn pool_pda_seed(mint: &[u8; 32]) -> PdaSeed<'_> {
    PdaSeed::new(mint)
}

#[inline]
pub const fn pool_pda_signer<'a>(mint: &'a [u8; 32], bump: &'a u8) -> [PdaSeed<'a>; 2] {
    let s0 = pool_pda_seed(mint);
    [s0, PdaSeed::new(slice::from_ref(bump))]
}

#[inline]
pub const fn wsol_bridge_seeds(mint: &[u8; 32]) -> [PdaSeed<'_>; 2] {
    let (s0, s1) = wsol_bridge_pda_seeds(mint);
    [PdaSeed::new(s0), PdaSeed::new(s1)]
}

#[inline]
pub const fn portfolio_seeds(mint: &[u8; 32]) -> [PdaSeed<'_>; 2] {
    let (s0, s1) = portfolio_pda_seeds(mint);
    [PdaSeed::new(s0), PdaSeed::new(s1)]
}

#[inline]
pub const fn wsol_bridge_signer<'a>(mint: &'a [u8; 32], bump: &'a u8) -> [PdaSeed<'a>; 3] {
    let [s0, s1] = wsol_bridge_seeds(mint);
    [s0, s1, PdaSeed::new(slice::from_ref(bump))]
}

#[inline]
pub const fn portfolio_signer<'a>(mint: &'a [u8; 32], bump: &'a u8) -> [PdaSeed<'a>; 3] {
    let [s0, s1] = portfolio_seeds(mint);
    [s0, s1, PdaSeed::new(slice::from_ref(bump))]
}

/// ~200 CUs
#[inline]
pub fn create_raw_pool_pda_to<'dst>(
    mint: &[u8; 32],
    bump: &u8,
    to: &'dst mut MaybeUninit<[u8; 32]>,
) -> Result<&'dst mut [u8; 32], InvalidKnownPdaErr> {
    let s0 = pool_pda_seed(mint);
    let seeds = [
        s0,
        PdaSeed::new(core::slice::from_ref(bump)),
        PdaSeed::new(CONST_KEYS_OWNED.program_id()),
        PdaSeed::new(&PDA_MARKER),
    ];
    // using `create_raw_program_address_to` over
    // `create_raw_program_address` saves 12 CUs
    create_raw_program_address_to(&seeds, to).ok_or(InvalidKnownPdaErr::POOL)
}

/// [`create_raw_pool_pda`] but with err converted for easier interop
/// with [`jiminy_program_error::ProgramError`]
#[inline]
pub fn prog_create_raw_pool_pda_to<'dst>(
    mint: &[u8; 32],
    bump: &u8,
    to: &'dst mut MaybeUninit<[u8; 32]>,
) -> Result<&'dst mut [u8; 32], SanctumSolsProgErr> {
    create_raw_pool_pda_to(mint, bump, to)
        .map_err(SanctumSolsErr::InvalidPda)
        .map_err(SanctumSolsProgErr)
}

#[inline]
pub fn create_raw_portfolio_pda_to<'dst>(
    mint: &[u8; 32],
    bump: &u8,
    to: &'dst mut MaybeUninit<[u8; 32]>,
) -> Result<&'dst mut [u8; 32], InvalidKnownPdaErr> {
    let [s0, s1] = portfolio_seeds(mint);
    let seeds = [
        s0,
        s1,
        PdaSeed::new(core::slice::from_ref(bump)),
        PdaSeed::new(CONST_KEYS_OWNED.program_id()),
        PdaSeed::new(&PDA_MARKER),
    ];
    create_raw_program_address_to(&seeds, to).ok_or(InvalidKnownPdaErr::PORTFOLIO)
}

#[inline]
pub fn prog_create_raw_portfolio_pda_to<'dst>(
    mint: &[u8; 32],
    bump: &u8,
    to: &'dst mut MaybeUninit<[u8; 32]>,
) -> Result<&'dst mut [u8; 32], SanctumSolsProgErr> {
    create_raw_portfolio_pda_to(mint, bump, to)
        .map_err(SanctumSolsErr::InvalidPda)
        .map_err(SanctumSolsProgErr)
}

#[inline]
pub fn create_raw_wsol_bridge_to<'dst>(
    mint: &[u8; 32],
    bump: &u8,
    to: &'dst mut MaybeUninit<[u8; 32]>,
) -> Result<&'dst mut [u8; 32], InvalidKnownPdaErr> {
    let [s0, s1] = wsol_bridge_seeds(mint);
    let seeds = [
        s0,
        s1,
        PdaSeed::new(core::slice::from_ref(bump)),
        PdaSeed::new(CONST_KEYS_OWNED.program_id()),
        PdaSeed::new(&PDA_MARKER),
    ];
    // using `create_raw_program_address_to` over
    // `create_raw_program_address` saves 12 CUs
    create_raw_program_address_to(&seeds, to).ok_or(InvalidKnownPdaErr::POOL)
}

/// [`create_raw_wsol_bridge_to`] but with err converted for easier interop
/// with [`jiminy_program_error::ProgramError`]
#[inline]
pub fn prog_create_wsol_bridge_to<'dst>(
    mint: &[u8; 32],
    bump: &u8,
    to: &'dst mut MaybeUninit<[u8; 32]>,
) -> Result<&'dst mut [u8; 32], SanctumSolsProgErr> {
    create_raw_wsol_bridge_to(mint, bump, to)
        .map_err(SanctumSolsErr::InvalidPda)
        .map_err(SanctumSolsProgErr)
}

#[inline]
fn try_find_pda(
    seeds: &[PdaSeed],
    err: InvalidKnownPdaErr,
) -> Result<([u8; 32], u8), InvalidKnownPdaErr> {
    try_find_program_address(seeds, CONST_KEYS_OWNED.program_id()).ok_or(err)
}

#[inline]
pub fn find_pool_pda(mint: &[u8; 32]) -> Result<([u8; 32], u8), InvalidKnownPdaErr> {
    let s0 = pool_pda_seed(mint);
    try_find_pda(&[s0], InvalidKnownPdaErr::POOL)
}

#[inline]
pub fn prog_find_pool_pda(mint: &[u8; 32]) -> Result<([u8; 32], u8), SanctumSolsProgErr> {
    find_pool_pda(mint)
        .map_err(SanctumSolsErr::InvalidPda)
        .map_err(SanctumSolsProgErr)
}

#[inline]
pub fn find_wsol_bridge_pda(mint: &[u8; 32]) -> Result<([u8; 32], u8), InvalidKnownPdaErr> {
    let [s0, s1] = wsol_bridge_seeds(mint);
    try_find_pda(&[s0, s1], InvalidKnownPdaErr::WSOL_BRIDGE)
}

#[inline]
pub fn prog_find_wsol_bridge_pda(mint: &[u8; 32]) -> Result<([u8; 32], u8), SanctumSolsProgErr> {
    find_wsol_bridge_pda(mint)
        .map_err(SanctumSolsErr::InvalidPda)
        .map_err(SanctumSolsProgErr)
}

#[inline]
pub fn find_portfolio_pda(mint: &[u8; 32]) -> Result<([u8; 32], u8), InvalidKnownPdaErr> {
    let [s0, s1] = portfolio_seeds(mint);
    try_find_pda(&[s0, s1], InvalidKnownPdaErr::PORTFOLIO)
}

#[inline]
pub fn prog_find_portfolio_pda(mint: &[u8; 32]) -> Result<([u8; 32], u8), SanctumSolsProgErr> {
    find_portfolio_pda(mint)
        .map_err(SanctumSolsErr::InvalidPda)
        .map_err(SanctumSolsProgErr)
}
