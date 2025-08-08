use inf1_pp_ag_core::PricingAg;
use inf1_pp_flatfee_std::update::mint_lp::PkIter as FlatFeePkIter;

use crate::PricingProgAg;

// Re-exports
pub use inf1_pp_std::update::AccountsToUpdateMintLp;
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
