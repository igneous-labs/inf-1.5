#![cfg_attr(not(test), no_std)]

use crate::keys::{CONST_KEYS_OWNED, CONST_KEY_STRS};

mod internal_utils;

pub mod accounts;
pub mod err;
pub mod instructions;
pub mod keys;
pub mod pda;
pub mod svc;
pub mod sync_sol_val;
pub mod token_info;
pub mod typedefs;
pub mod yields;

#[deprecated = "Use `CONST_KEY_STRS.program()` instead"]
pub const ID_STR: &str = CONST_KEY_STRS.program();

#[deprecated = "Use `*CONST_KEYS_OWNED.program()` instead"]
pub const ID: [u8; 32] = *CONST_KEYS_OWNED.program();
