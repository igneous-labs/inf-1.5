#![cfg_attr(not(test), no_std)]

// Re-exports
pub use inf1_svc_core;

pub mod calc;
pub mod instructions;

pub const ID_STR: &str = "wsoGmxQLSvwWpuaidCApxN5kEowLe2HLQLJhCQnj4bE";
pub const ID: [u8; 32] = const_crypto::bs58::decode_pubkey(ID_STR);
