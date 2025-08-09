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

pub trait AccountsToUpdateRedeemLp {
    type PkIter: Iterator<Item = [u8; 32]>;

    /// Returned iterator can yield duplicate pubkeys,
    /// responsibility of caller to dedup if required
    fn accounts_to_update_redeem_lp(&self) -> Self::PkIter;
}

pub trait AccountsToUpdatePriceExactIn {
    type PkIter: Iterator<Item = [u8; 32]>;

    /// Returned iterator can yield duplicate pubkeys,
    /// responsibility of caller to dedup if required
    fn accounts_to_update_price_exact_in(&self, swap_mints: &Pair<&[u8; 32]>) -> Self::PkIter;
}

pub trait AccountsToUpdatePriceExactOut {
    type PkIter: Iterator<Item = [u8; 32]>;

    /// Returned iterator can yield duplicate pubkeys,
    /// responsibility of caller to dedup if required
    fn accounts_to_update_price_exact_out(&self, swap_mints: &Pair<&[u8; 32]>) -> Self::PkIter;
}

pub trait AccountsToUpdateAll {
    type PkIter: Iterator<Item = [u8; 32]>;

    /// Returns all the accounts from which this pricing program derives its data from
    /// (across all LSTs + Mint/Redeem LP) to fetch from onchain. Upon updating with
    /// the fetched accounts, this struct should be able to give the most up-to-date
    /// quote and ix accounts for all 4 interface functionalities.
    ///
    /// Iterator of mints it passed as arg in order to enable
    /// struct to fetch accounts required for new mints that were not
    /// previously known.
    ///
    /// Returned iterator can yield duplicate pubkeys,
    /// responsibility of caller to dedup if required
    fn accounts_to_update_all<'a>(
        &self,
        all_mints: impl IntoIterator<Item = &'a [u8; 32]>,
    ) -> Self::PkIter;
}

// Q: Why are the `AccountsToUpdate*` traits split into 4 but this `UpdatePricingProg` one is merged into one?
// A: This is to force all 4 update procedures to use the same `InnerErr` type in order to keep the # of different
//    err types low. OTOH, since pricing programs may have different sets of accounts needed for update for different procedures,
//    the `AccountsToUpdate*` traits are split up.
pub trait UpdatePricingProg {
    type InnerErr: Error;

    fn update_mint_lp(
        &mut self,
        update_map: impl UpdateMap,
    ) -> Result<(), UpdateErr<Self::InnerErr>>;

    fn update_program_state(
        &mut self,
        update_map: impl UpdateMap,
    ) -> Result<(), UpdateErr<Self::InnerErr>>;

    fn update_price_exact_in(
        &mut self,
        swap_mints: &Pair<&[u8; 32]>,
        update_map: impl UpdateMap,
    ) -> Result<(), UpdateErr<Self::InnerErr>>;

    fn update_price_exact_out(
        &mut self,
        swap_mints: &Pair<&[u8; 32]>,
        update_map: impl UpdateMap,
    ) -> Result<(), UpdateErr<Self::InnerErr>>;

    fn update_all<'a>(
        &mut self,
        all_mints: impl IntoIterator<Item = &'a [u8; 32]>,
        update_map: impl UpdateMap,
    ) -> Result<(), UpdateErr<Self::InnerErr>>;
}
