use inf1_pp_core::pair::Pair;

use crate::{
    update::{common::SwapPkIter, FlatFeePricingUpdateErr},
    FlatFeePricing,
};

// Re-exports
pub use inf1_pp_std::update::{
    AccountsToUpdatePriceExactOut, UpdateErr, UpdateMap, UpdatePriceExactOut,
};

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

pub type InnerErr = FlatFeePricingUpdateErr;

impl<
        F: Fn(&[&[u8]], &[u8; 32]) -> Option<([u8; 32], u8)>,
        C: Fn(&[&[u8]], &[u8; 32]) -> Option<[u8; 32]>,
    > UpdatePriceExactOut for FlatFeePricing<F, C>
{
    type InnerErr = InnerErr;

    #[inline]
    fn update_price_exact_out(
        &mut self,
        swap_mints: &Pair<&[u8; 32]>,
        update_map: impl UpdateMap,
    ) -> Result<(), UpdateErr<Self::InnerErr>> {
        self.update_swap_pair(swap_mints, update_map)
    }
}
