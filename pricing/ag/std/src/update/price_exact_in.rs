use inf1_pp_ag_core::PricingAg;
use inf1_pp_flatfee_std::update::price_exact_in::PkIter as FlatFeePkIter;
use inf1_pp_flatslab_std::update::PkIter as FlatSlabPkIter;

use crate::{internal_utils::map_variant_method, PricingProgAg};

// Re-exports
pub use inf1_pp_std::{pair::Pair, update::AccountsToUpdatePriceExactIn};
pub use inf1_update_traits::{UpdateErr, UpdateMap};

pub type PkIter = PricingAg<FlatFeePkIter, FlatSlabPkIter>;

impl<
        F: Fn(&[&[u8]], &[u8; 32]) -> Option<([u8; 32], u8)>,
        C: Fn(&[&[u8]], &[u8; 32]) -> Option<[u8; 32]>,
    > AccountsToUpdatePriceExactIn for PricingProgAg<F, C>
{
    type PkIter = PkIter;

    #[inline]
    fn accounts_to_update_price_exact_in(&self, swap_mints: &Pair<&[u8; 32]>) -> Self::PkIter {
        map_variant_method!(&self.0, accounts_to_update_price_exact_in(swap_mints))
    }
}
