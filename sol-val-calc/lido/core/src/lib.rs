#![cfg_attr(not(test), no_std)]

// Re-exports
pub use inf1_svc_core;
pub use inf1_svc_generic;
pub use solido_legacy_core;

pub mod calc;
pub mod instructions;
pub mod keys;

keys::id_str!(ID_STR, ID, "1idUSy4MGGKyKhvjSnGZ6Zc7Q4eKQcibym4BkEEw9KR");
