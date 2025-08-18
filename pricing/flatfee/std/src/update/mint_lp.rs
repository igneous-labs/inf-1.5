use std::iter::{empty, Empty};

use crate::FlatFeePricing;

// Re-exports
pub use inf1_pp_std::update::{AccountsToUpdateMintLp, UpdateErr, UpdateMap};

pub type PkIter = Empty<[u8; 32]>;

impl<F, C> AccountsToUpdateMintLp for FlatFeePricing<F, C> {
    type PkIter = PkIter;

    #[inline]
    fn accounts_to_update_mint_lp(&self, _inp_mint: &[u8; 32]) -> Self::PkIter {
        empty()
    }
}
