use inf1_pp_ag_core::PricingAg;
use inf1_pp_flatfee_std::update::all::PkIter as FlatFeePkIter;

use crate::PricingProgAg;

// Re-exports
pub use inf1_pp_std::{pair::Pair, update::AccountsToUpdateAll};
pub use inf1_update_traits::{UpdateErr, UpdateMap};

pub type PkIter = PricingAg<FlatFeePkIter>;

impl<
        F: Fn(&[&[u8]], &[u8; 32]) -> Option<([u8; 32], u8)>,
        C: Fn(&[&[u8]], &[u8; 32]) -> Option<[u8; 32]>,
    > AccountsToUpdateAll for PricingProgAg<F, C>
{
    type PkIter = PkIter;

    fn accounts_to_update_all<'a>(
        &self,
        all_mints: impl IntoIterator<Item = &'a [u8; 32]>,
    ) -> Self::PkIter {
        match &self.0 {
            PricingAg::FlatFee(p) => PricingAg::FlatFee(p.accounts_to_update_all(all_mints)),
        }
    }
}
