use std::convert::Infallible;

use inf1_pp_ag_core::PricingAg;
use inf1_pp_flatfee_std::{traits::FlatFeePricingColErr, FlatFeePricing};

pub mod traits;

pub type FindPdaFnPtr = fn(&[&[u8]], &[u8; 32]) -> Option<([u8; 32], u8)>;

pub type CreatePdaFnPtr = fn(&[&[u8]], &[u8; 32]) -> Option<[u8; 32]>;

// simple newtype to workaround orphan rules
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PricingProgAg<F, C>(pub PricingAg<FlatFeePricing<F, C>>);

pub type PricingProgAgStd = PricingProgAg<FindPdaFnPtr, CreatePdaFnPtr>;

pub type PricingProgAgErr = PricingAg<FlatFeePricingColErr>;

pub type PricingProgAgInfallibleErr = PricingAg<Infallible>;
