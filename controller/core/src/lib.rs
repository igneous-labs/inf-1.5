#![cfg_attr(not(test), no_std)]

mod internal_utils;

pub mod accounts;
pub mod instructions;
pub mod keys;
pub mod pda;
pub mod typedefs;

keys::id_str!(ID_STR, ID, "5ocnV1qiCgaQR8Jb8xWnVbApfaygJ8tNoZfgPwsgx9kx");
