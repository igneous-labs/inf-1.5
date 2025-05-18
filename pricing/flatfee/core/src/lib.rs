#![cfg_attr(not(test), no_std)]

// Re-exports
pub use inf1_pricing_core;

pub mod accounts;
pub mod instructions;
pub mod keys;
pub mod pda;
pub mod pricing;

keys::id_str!(ID_STR, ID, "f1tUoNEKrDp1oeGn4zxr7bh41eN6VcfHjfrL3ZqQday");
