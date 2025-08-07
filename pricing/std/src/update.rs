// TODO: deprecate the *Lp traits

use std::error::Error;

use inf1_pp_core::pair::Pair;
// Re-exports
pub use inf1_update_traits::*;

// TODO: combine 4-Update* traits into 1 trait, else there are too many different err types flying around.

pub trait AccountsToUpdateMintLp {
    type PkIter: Iterator<Item = [u8; 32]>;

    /// Returned iterator can yield duplicate pubkeys,
    /// responsibility of caller to dedup if required
    fn accounts_to_update_mint_lp(&self) -> Self::PkIter;
}

pub trait UpdateMintLp {
    type InnerErr: Error;

    fn update_mint_lp(
        &mut self,
        update_map: impl UpdateMap,
    ) -> Result<(), UpdateErr<Self::InnerErr>>;
}

pub trait AccountsToUpdateRedeemLp {
    type PkIter: Iterator<Item = [u8; 32]>;

    /// Returned iterator can yield duplicate pubkeys,
    /// responsibility of caller to dedup if required
    fn accounts_to_update_redeem_lp(&self) -> Self::PkIter;
}

pub trait UpdateRedeemLp {
    type InnerErr: Error;

    fn update_redeem_lp(
        &mut self,
        update_map: impl UpdateMap,
    ) -> Result<(), UpdateErr<Self::InnerErr>>;
}

pub trait AccountsToUpdatePriceExactIn {
    type PkIter: Iterator<Item = [u8; 32]>;

    /// Returned iterator can yield duplicate pubkeys,
    /// responsibility of caller to dedup if required
    fn accounts_to_update_price_exact_in(&self, swap_mints: &Pair<&[u8; 32]>) -> Self::PkIter;
}

pub trait UpdatePriceExactIn {
    type InnerErr: Error;

    fn update_price_exact_in(
        &mut self,
        swap_mints: &Pair<&[u8; 32]>,
        update_map: impl UpdateMap,
    ) -> Result<(), UpdateErr<Self::InnerErr>>;
}

pub trait AccountsToUpdatePriceExactOut {
    type PkIter: Iterator<Item = [u8; 32]>;

    /// Returned iterator can yield duplicate pubkeys,
    /// responsibility of caller to dedup if required
    fn accounts_to_update_price_exact_out(&self, swap_mints: &Pair<&[u8; 32]>) -> Self::PkIter;
}

pub trait UpdatePriceExactOut {
    type InnerErr: Error;

    fn update_price_exact_out(
        &mut self,
        swap_mints: &Pair<&[u8; 32]>,
        update_map: impl UpdateMap,
    ) -> Result<(), UpdateErr<Self::InnerErr>>;
}
