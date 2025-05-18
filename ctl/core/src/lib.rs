#![cfg_attr(not(test), no_std)]

pub mod instructions;
pub mod keys;
pub mod pda;

keys::id_str!(ID_STR, ID, "5ocnV1qiCgaQR8Jb8xWnVbApfaygJ8tNoZfgPwsgx9kx");
