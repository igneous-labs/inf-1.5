use inf1_pp_core::pair::Pair;

use crate::{
    update::{common::SwapPkIter, FlatFeePricingUpdateErr},
    FlatFeePricing,
};

// Re-exports
pub use inf1_pp_std::update::{
    AccountsToUpdatePriceExactIn, UpdateErr, UpdateMap, UpdatePriceExactIn,
};

pub type PkIter = SwapPkIter;

impl<
        F: Fn(&[&[u8]], &[u8; 32]) -> Option<([u8; 32], u8)>,
        C: Fn(&[&[u8]], &[u8; 32]) -> Option<[u8; 32]>,
    > AccountsToUpdatePriceExactIn for FlatFeePricing<F, C>
{
    type PkIter = PkIter;

    #[inline]
    fn accounts_to_update_price_exact_in(&self, pair: &Pair<&[u8; 32]>) -> Self::PkIter {
        self.accounts_to_update_swap_pair(pair)
    }
}

pub type InnerErr = FlatFeePricingUpdateErr;

impl<
        F: Fn(&[&[u8]], &[u8; 32]) -> Option<([u8; 32], u8)>,
        C: Fn(&[&[u8]], &[u8; 32]) -> Option<[u8; 32]>,
    > UpdatePriceExactIn for FlatFeePricing<F, C>
{
    type InnerErr = InnerErr;

    #[inline]
    fn update_price_exact_in(
        &mut self,
        swap_mints: &Pair<&[u8; 32]>,
        update_map: impl UpdateMap,
    ) -> Result<(), UpdateErr<Self::InnerErr>> {
        self.update_swap_pair(swap_mints, update_map)
    }
}
