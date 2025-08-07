use inf1_pp_flatfee_std::{traits::FlatFeePricingColErr, FlatFeePricing};

// Re-exports
pub use inf1_pp_ag_core::*;
pub use inf1_pp_flatfee_std;

pub mod traits;
pub mod update;

pub type FindPdaFnPtr = fn(&[&[u8]], &[u8; 32]) -> Option<([u8; 32], u8)>;

pub type CreatePdaFnPtr = fn(&[&[u8]], &[u8; 32]) -> Option<[u8; 32]>;

// simple newtype to workaround orphan rules
#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct PricingProgAg<F, C>(pub PricingAg<FlatFeePricing<F, C>>);

pub type PricingProgAgStd = PricingProgAg<FindPdaFnPtr, CreatePdaFnPtr>;

pub type PricingProgAgErr = PricingAg<FlatFeePricingColErr>;
