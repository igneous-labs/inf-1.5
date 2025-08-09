use std::{iter::once, vec};

use crate::FlatFeePricing;

// Re-exports
pub use inf1_pp_std::update::{AccountsToUpdateAll, UpdateErr, UpdateMap};

// Very unfortunate but we have to use a Vec intermediary.
// Cant use closures in associated types yet,
// + need to introduce new generic into trait to make use of
// <all_mints as IntoIterator>::IntoIter type
pub type PkIter = vec::IntoIter<[u8; 32]>;

impl<
        F: Fn(&[&[u8]], &[u8; 32]) -> Option<([u8; 32], u8)>,
        C: Fn(&[&[u8]], &[u8; 32]) -> Option<[u8; 32]>,
    > AccountsToUpdateAll for FlatFeePricing<F, C>
{
    type PkIter = PkIter;

    #[inline]
    fn accounts_to_update_all<'a>(
        &self,
        all_mints: impl IntoIterator<Item = &'a [u8; 32]>,
    ) -> Self::PkIter {
        all_mints
            .into_iter()
            .map(|mint| self.fee_account_pda(mint))
            .chain(once(inf1_pp_flatfee_core::keys::STATE_ID))
            .collect::<Vec<_>>()
            .into_iter()
    }
}
