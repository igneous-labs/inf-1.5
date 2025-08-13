#![cfg_attr(not(test), no_std)]

pub mod accounts;
pub mod errs;
pub mod instructions;
pub mod keys;
pub mod pda;
pub mod pricing;
pub mod typedefs;

mod internal_utils;

pub const ID_STR: &str = "s1b6NRXj6ygNu1QMKXh2H9LUR2aPApAAm1UQ2DjdhNV";
pub const ID: [u8; 32] = const_crypto::bs58::decode_pubkey(ID_STR);
