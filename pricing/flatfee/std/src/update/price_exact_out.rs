use inf1_pp_std::pair::Pair;

use crate::{update::common::SwapPkIter, FlatFeePricing};

// Re-exports
pub use inf1_pp_std::update::{AccountsToUpdatePriceExactOut, UpdateErr, UpdateMap};

pub type PkIter = SwapPkIter;

impl<
        F: Fn(&[&[u8]], &[u8; 32]) -> Option<([u8; 32], u8)>,
        C: Fn(&[&[u8]], &[u8; 32]) -> Option<[u8; 32]>,
    > AccountsToUpdatePriceExactOut for FlatFeePricing<F, C>
{
    type PkIter = PkIter;

    #[inline]
    fn accounts_to_update_price_exact_out(&self, pair: &Pair<&[u8; 32]>) -> Self::PkIter {
        self.accounts_to_update_swap_pair(pair)
    }
}
