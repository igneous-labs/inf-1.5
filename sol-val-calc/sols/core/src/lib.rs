#![cfg_attr(not(test), no_std)]

// Re-exports
pub use crate::keys::{ID, ID_STR};
pub use inf1_svc_core;
pub use inf1_svc_generic;
pub use sanctum_sols_core;

pub mod calc;
pub mod instructions;
pub mod keys;
