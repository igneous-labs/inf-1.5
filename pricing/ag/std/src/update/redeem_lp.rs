use inf1_pp_ag_core::PricingAg;
use inf1_pp_flatfee_std::update::redeem_lp::{
    InnerErr as FlatFeeInnerErr, PkIter as FlatFeePkIter,
};

use crate::PricingProgAg;

// Re-exports
pub use inf1_pp_std::update::{AccountsToUpdateRedeemLp, UpdateRedeemLp};
pub use inf1_update_traits::{UpdateErr, UpdateMap};

pub type PkIter = PricingAg<FlatFeePkIter>;

impl<F, C> AccountsToUpdateRedeemLp for PricingProgAg<F, C> {
    type PkIter = PkIter;

    #[inline]
    fn accounts_to_update_redeem_lp(&self) -> Self::PkIter {
        match &self.0 {
            PricingAg::FlatFee(p) => PricingAg::FlatFee(p.accounts_to_update_redeem_lp()),
        }
    }
}

pub type InnerErr = PricingAg<FlatFeeInnerErr>;

impl<F, C> UpdateRedeemLp for PricingProgAg<F, C> {
    type InnerErr = InnerErr;

    #[inline]
    fn update_redeem_lp(
        &mut self,
        update_map: impl UpdateMap,
    ) -> Result<(), UpdateErr<Self::InnerErr>> {
        match &mut self.0 {
            PricingAg::FlatFee(p) => p
                .update_redeem_lp(update_map)
                .map_err(|e| e.map_inner(PricingAg::FlatFee)),
        }
    }
}
