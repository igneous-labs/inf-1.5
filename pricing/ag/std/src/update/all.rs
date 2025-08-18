use inf1_pp_ag_core::PricingAg;
use inf1_pp_flatfee_std::update::all::PkIter as FlatFeePkIter;
use inf1_pp_flatslab_std::update::PkIter as FlatSlabPkIter;

use crate::{internal_utils::map_variant_method, PricingProgAg};

// Re-exports
pub use inf1_pp_std::{pair::Pair, update::AccountsToUpdateAll};
pub use inf1_update_traits::{UpdateErr, UpdateMap};

pub type PkIter = PricingAg<FlatFeePkIter, FlatSlabPkIter>;

impl<
        F: Fn(&[&[u8]], &[u8; 32]) -> Option<([u8; 32], u8)>,
        C: Fn(&[&[u8]], &[u8; 32]) -> Option<[u8; 32]>,
    > AccountsToUpdateAll for PricingProgAg<F, C>
{
    type PkIter = PkIter;

    fn accounts_to_update_all(
        &self,
        all_mints: impl IntoIterator<Item = [u8; 32]>,
    ) -> Self::PkIter {
        map_variant_method!(&self.0, accounts_to_update_all(all_mints))
    }
}
