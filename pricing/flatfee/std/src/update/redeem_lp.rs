use std::iter::{once, Once};

use crate::FlatFeePricing;

// Re-exports
pub use inf1_pp_std::update::{Account, AccountsToUpdateRedeemLp, UpdateErr, UpdateMap};

pub type PkIter = Once<[u8; 32]>;

impl<F, C> AccountsToUpdateRedeemLp for FlatFeePricing<F, C> {
    type PkIter = PkIter;

    #[inline]
    fn accounts_to_update_redeem_lp(&self, _out_mint: &[u8; 32]) -> Self::PkIter {
        once(inf1_pp_flatfee_core::keys::STATE_ID)
    }
}

impl<F, C> FlatFeePricing<F, C> {
    #[inline]
    pub const fn update_lp_withdrawal_fee_bps(&mut self, lp_withdrawal_fee_bps: u16) {
        self.lp_withdrawal_fee_bps = Some(lp_withdrawal_fee_bps);
    }
}
