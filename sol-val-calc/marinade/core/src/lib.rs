#![cfg_attr(not(test), no_std)]

// Re-exports
pub use inf1_svc_core;
pub use inf1_svc_generic;
pub use sanctum_marinade_liquid_staking_core;

pub mod calc;
pub mod instructions;
pub mod keys;

keys::id_str!(ID_STR, ID, "mare3SCyfZkAndpBRBeonETmkCCB3TJTTrz8ZN2dnhP");
