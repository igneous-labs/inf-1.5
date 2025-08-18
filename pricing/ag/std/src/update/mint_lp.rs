use inf1_pp_ag_core::PricingAg;
use inf1_pp_flatfee_std::update::mint_lp::PkIter as FlatFeePkIter;
use inf1_pp_flatslab_std::update::PkIter as FlatSlabPkIter;

use crate::{internal_utils::map_variant_method, PricingProgAg};

// Re-exports
pub use inf1_pp_std::update::AccountsToUpdateMintLp;
pub use inf1_update_traits::{UpdateErr, UpdateMap};

pub type PkIter = PricingAg<FlatFeePkIter, FlatSlabPkIter>;

impl<F, C> AccountsToUpdateMintLp for PricingProgAg<F, C> {
    type PkIter = PkIter;

    #[inline]
    fn accounts_to_update_mint_lp(&self, inp_mint: &[u8; 32]) -> Self::PkIter {
        map_variant_method!(&self.0, accounts_to_update_mint_lp(inp_mint))
    }
}
