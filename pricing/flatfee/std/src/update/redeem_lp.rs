use std::iter::{once, Once};

use inf1_pp_flatfee_core::accounts::program_state::{ProgramState, ProgramStatePacked};

use crate::{update::FlatFeePricingUpdateErr, FlatFeePricing};

// Re-exports
pub use inf1_pp_std::update::{
    Account, AccountsToUpdateRedeemLp, UpdateErr, UpdateMap, UpdateRedeemLp,
};

pub type PkIter = Once<[u8; 32]>;

impl<F, C> AccountsToUpdateRedeemLp for FlatFeePricing<F, C> {
    type PkIter = PkIter;

    #[inline]
    fn accounts_to_update_redeem_lp(&self) -> Self::PkIter {
        once(inf1_pp_flatfee_core::keys::STATE_ID)
    }
}

impl<F, C> FlatFeePricing<F, C> {
    #[inline]
    pub const fn update_lp_withdrawal_fee_bps(&mut self, lp_withdrawal_fee_bps: u16) {
        self.lp_withdrawal_fee_bps = Some(lp_withdrawal_fee_bps);
    }
}

pub type InnerErr = FlatFeePricingUpdateErr;

impl<F, C> UpdateRedeemLp for FlatFeePricing<F, C> {
    type InnerErr = InnerErr;

    #[inline]
    fn update_redeem_lp(
        &mut self,
        update_map: impl UpdateMap,
    ) -> Result<(), UpdateErr<Self::InnerErr>> {
        let new_program_state =
            update_map.get_account_checked(&inf1_pp_flatfee_core::keys::STATE_ID)?;
        let ProgramState {
            lp_withdrawal_fee_bps,
            ..
        } = ProgramStatePacked::of_acc_data(new_program_state.data())
            .ok_or(UpdateErr::Inner(InnerErr::AccDeser {
                pk: inf1_pp_flatfee_core::keys::STATE_ID,
            }))?
            .into_program_state();

        self.update_lp_withdrawal_fee_bps(lp_withdrawal_fee_bps);

        Ok(())
    }
}
