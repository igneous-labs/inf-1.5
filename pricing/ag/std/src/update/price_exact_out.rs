use inf1_pp_ag_core::PricingAg;
use inf1_pp_flatfee_std::update::price_exact_out::PkIter as FlatFeePkIter;

use crate::PricingProgAg;

// Re-exports
pub use inf1_pp_std::{pair::Pair, update::AccountsToUpdatePriceExactOut};
pub use inf1_update_traits::{UpdateErr, UpdateMap};

pub type PkIter = PricingAg<FlatFeePkIter>;

impl<
        F: Fn(&[&[u8]], &[u8; 32]) -> Option<([u8; 32], u8)>,
        C: Fn(&[&[u8]], &[u8; 32]) -> Option<[u8; 32]>,
    > AccountsToUpdatePriceExactOut for PricingProgAg<F, C>
{
    type PkIter = PkIter;

    #[inline]
    fn accounts_to_update_price_exact_out(&self, swap_mints: &Pair<&[u8; 32]>) -> Self::PkIter {
        match &self.0 {
            PricingAg::FlatFee(p) => {
                PricingAg::FlatFee(p.accounts_to_update_price_exact_out(swap_mints))
            }
        }
    }
}
