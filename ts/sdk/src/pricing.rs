use std::collections::HashMap;

use bs58_fixed_wasm::Bs58Array;
use inf1_pp_flatfee_std::accounts::{
    fee::FeeAccountPacked,
    program_state::{ProgramState, ProgramStatePacked},
};

use crate::{
    err::{acc_deser_err, missing_acc_err, InfError},
    interface::{Account, B58PK},
    pda::{create_raw_pda_slice, find_pda},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlatFeePricing(pub inf1_pp_flatfee_std::FlatFeePricingStd);

impl Default for FlatFeePricing {
    fn default() -> Self {
        Self(inf1_pp_flatfee_std::FlatFeePricingStd::new(
            None,
            Default::default(),
            find_pda,
            create_raw_pda_slice,
        ))
    }
}

// TODO: find a better way to generalize accounts + update fn for aggregations

/// Accounts To Update
impl FlatFeePricing {
    pub(crate) fn account_to_update_remove_liquidity(&self) -> [u8; 32] {
        self.0.account_to_update_remove_liquidity()
    }

    pub(crate) fn accounts_to_update_swap<'a, I: IntoIterator<Item = &'a [u8; 32]>>(
        &self,
        mints: I,
    ) -> impl Iterator<Item = [u8; 32]> + use<'a, '_, I> {
        self.0.accounts_to_update_swap(mints)
    }
}

/// Update
impl FlatFeePricing {
    #[inline]
    pub fn update_remove_liquidity(
        &mut self,
        fetched: &HashMap<B58PK, Account>,
    ) -> Result<(), InfError> {
        let new_program_state = fetched
            .get(&Bs58Array(inf1_pp_flatfee_std::keys::STATE_ID))
            .ok_or_else(|| missing_acc_err(&inf1_pp_flatfee_std::keys::STATE_ID))?;
        let ProgramState {
            lp_withdrawal_fee_bps,
            ..
        } = ProgramStatePacked::of_acc_data(&new_program_state.data)
            .ok_or_else(|| acc_deser_err(&inf1_pp_flatfee_std::keys::STATE_ID))?
            .into_program_state();

        self.0.update_lp_withdrawal_fee_bps(lp_withdrawal_fee_bps);

        Ok(())
    }

    #[inline]
    pub fn update_swap<'a, I: IntoIterator<Item = &'a [u8; 32]>>(
        &mut self,
        mints: I,
        fetched: &HashMap<B58PK, Account>,
    ) -> Result<(), InfError> {
        mints.into_iter().try_for_each(|mint| {
            let fee_acc = self.0.fee_account_pda(mint);
            let new_fee_acc = fetched
                .get(&Bs58Array(fee_acc))
                .ok_or_else(|| missing_acc_err(&fee_acc))?;
            let new_fee_acc = FeeAccountPacked::of_acc_data(&new_fee_acc.data)
                .ok_or_else(|| acc_deser_err(&fee_acc))?
                .into_fee_account();

            self.0.upsert_fee_account(*mint, new_fee_acc);

            Ok(())
        })
    }
}
