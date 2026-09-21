use inf1_pp_flatfee_std::{traits::FlatFeePricingColErr, FlatFeePricing};
use inf1_pp_flatslab_std::{traits::FlatSlabPricingColErr, FlatSlabPricing};
use inf1_pp_reserve_v2_std::{ReserveV2Pricing, ReserveV2PricingColErr};

// Re-exports
pub use inf1_pp_ag_core::*;
pub use inf1_pp_flatfee_std;
pub use inf1_pp_flatslab_std;
pub use inf1_pp_reserve_v2_std;

pub mod traits;
pub mod update;

mod internal_utils;

pub type FindPdaFnPtr = fn(&[&[u8]], &[u8; 32]) -> Option<([u8; 32], u8)>;

pub type CreatePdaFnPtr = fn(&[&[u8]], &[u8; 32]) -> Option<[u8; 32]>;

// simple newtype to workaround orphan rules
#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct PricingProgAg<F, C>(
    pub PricingAg<FlatFeePricing<F, C>, FlatSlabPricing, ReserveV2Pricing>,
);

pub type PricingProgAgStd = PricingProgAg<FindPdaFnPtr, CreatePdaFnPtr>;

pub type PricingProgAgErr =
    PricingAg<FlatFeePricingColErr, FlatSlabPricingColErr, ReserveV2PricingColErr>;
