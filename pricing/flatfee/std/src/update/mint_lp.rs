use std::{convert::Infallible, option};

use crate::FlatFeePricing;

// Re-exports
pub use inf1_pp_std::update::{AccountsToUpdateMintLp, UpdateErr, UpdateMap, UpdateMintLp};

pub type PkIter = option::IntoIter<[u8; 32]>;

impl<F, C> AccountsToUpdateMintLp for FlatFeePricing<F, C> {
    type PkIter = PkIter;

    #[inline]
    fn accounts_to_update_mint_lp(&self) -> Self::PkIter {
        None.into_iter()
    }
}

pub type InnerErr = Infallible;

impl<F, C> UpdateMintLp for FlatFeePricing<F, C> {
    type InnerErr = InnerErr;

    #[inline]
    fn update_mint_lp(
        &mut self,
        _update_map: impl UpdateMap,
    ) -> Result<(), UpdateErr<Self::InnerErr>> {
        Ok(())
    }
}
