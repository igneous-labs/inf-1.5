use std::convert::Infallible;

use inf1_pp_ag_core::PricingAg;
use inf1_pp_flatfee_std::update::mint_lp::PkIter as FlatFeePkIter;

use crate::PricingProgAg;

// Re-exports
pub use inf1_pp_std::update::{AccountsToUpdateMintLp, UpdateMintLp};
pub use inf1_update_traits::{UpdateErr, UpdateMap};

pub type PkIter = PricingAg<FlatFeePkIter>;

impl<F, C> AccountsToUpdateMintLp for PricingProgAg<F, C> {
    type PkIter = PkIter;

    #[inline]
    fn accounts_to_update_mint_lp(&self) -> Self::PkIter {
        match &self.0 {
            PricingAg::FlatFee(p) => PricingAg::FlatFee(p.accounts_to_update_mint_lp()),
        }
    }
}

pub type InnerErr = PricingAg<Infallible>;

impl<F, C> UpdateMintLp for PricingProgAg<F, C> {
    type InnerErr = InnerErr;

    #[inline]
    fn update_mint_lp(
        &mut self,
        update_map: impl UpdateMap,
    ) -> Result<(), UpdateErr<Self::InnerErr>> {
        match &mut self.0 {
            PricingAg::FlatFee(p) => p
                .update_mint_lp(update_map)
                .map_err(|e| e.map_inner(PricingAg::FlatFee)),
        }
    }
}
