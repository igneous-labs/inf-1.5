use inf1_pp_ag_std::{PricingAg, PricingAgTy, PricingProgAg, PricingProgAgStd};
use inf1_pp_flatfee_std::FlatFeePricingStd;

use crate::pda::{create_raw_pda_slice, find_pda};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlatFeePricing(pub inf1_pp_flatfee_std::FlatFeePricingStd);

impl Default for FlatFeePricing {
    fn default() -> Self {
        Self(inf1_pp_flatfee_std::FlatFeePricingStd::new(
            None,
            Default::default(),
            find_pda,
            create_raw_pda_slice,
        ))
    }
}

// TODO: find a better way to generalize accounts + update fn for aggregations

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pricing(pub PricingProgAgStd);

impl Default for Pricing {
    #[inline]
    fn default() -> Self {
        Self::flat_fee_default()
    }
}

/// Constructors
impl Pricing {
    #[inline]
    pub(crate) fn try_default_from_program_id(program_id: &[u8; 32]) -> Option<Self> {
        PricingAgTy::try_from_program_id(program_id).map(|ty| match ty {
            PricingAgTy::FlatFee => Self::flat_fee_default(),
        })
    }
}

/// default for variants
impl Pricing {
    #[inline]
    pub(crate) fn flat_fee_default() -> Self {
        Self(PricingProgAg(PricingAg::FlatFee(FlatFeePricingStd::new(
            None,
            Default::default(),
            find_pda,
            create_raw_pda_slice,
        ))))
    }
}
