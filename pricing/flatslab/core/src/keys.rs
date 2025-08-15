use const_crypto::{
    bs58::{decode_pubkey, encode_pubkey},
    ed25519::derive_program_address,
};

use crate::pda::SLAB_SEED;

pub const LP_MINT_ID_STR: &str = "5oVNBeEEQvYi1cX3ir8Dx5n1P7pdxydbGF2X4TxVusJm";
/// Hardcoded LP token mint (INF)
pub const LP_MINT_ID: [u8; 32] = decode_pubkey(LP_MINT_ID_STR);

const SLAB: ([u8; 32], u8) = derive_program_address(&[&SLAB_SEED], &crate::ID);
pub const SLAB_ID: [u8; 32] = SLAB.0;
pub const SLAB_BUMP: u8 = SLAB.1;
pub const SLAB_ID_STR: &str = encode_pubkey(&crate::ID).str();

pub const INITIAL_ADMIN_ID_STR: &str = "27L3WY8LMmrfAXoeFA7RP1d9ifu96YibMyXYPUieQqTD";
pub const INITIAL_ADMIN_ID: [u8; 32] = decode_pubkey(INITIAL_ADMIN_ID_STR);
