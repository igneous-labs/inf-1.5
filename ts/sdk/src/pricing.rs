use std::collections::HashMap;

use bs58_fixed_wasm::Bs58Array;
use inf1_pp_flatfee_core::{
    accounts::{
        fee::{FeeAccount, FeeAccountPacked},
        program_state::{ProgramState, ProgramStatePacked},
    },
    pda::fee_account_seeds,
    ID,
};
use wasm_bindgen::JsError;

use crate::{
    err::{acc_deser_err, missing_acc_err, no_valid_pda_err},
    interface::{Account, B58PK},
    pda::{create_raw_pda, find_pda},
};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct FlatFeePricing {
    pub lp_withdrawal_fee_bps: Option<u16>,

    /// key=mint
    pub lsts: HashMap<[u8; 32], FeeAccount>,
}

/// Update
impl FlatFeePricing {
    #[inline]
    pub const fn account_to_update_remove_liquidity(&self) -> [u8; 32] {
        inf1_pp_flatfee_core::keys::STATE_ID
    }

    #[inline]
    pub fn update_remove_liquidity(
        &mut self,
        fetched: &HashMap<B58PK, Account>,
    ) -> Result<(), JsError> {
        let new_program_state = fetched
            .get(&Bs58Array(inf1_pp_flatfee_core::keys::STATE_ID))
            .ok_or_else(|| missing_acc_err(&inf1_pp_flatfee_core::keys::STATE_ID))?;
        let ProgramState {
            lp_withdrawal_fee_bps,
            ..
        } = ProgramStatePacked::of_acc_data(&new_program_state.data)
            .ok_or_else(|| acc_deser_err(&inf1_pp_flatfee_core::keys::STATE_ID))?
            .into_program_state();

        self.lp_withdrawal_fee_bps = Some(lp_withdrawal_fee_bps);

        Ok(())
    }

    #[inline]
    pub fn fee_account(&self, mint: &[u8; 32]) -> Option<[u8; 32]> {
        self.lsts.get(mint).map_or_else(
            || find_fee_account_pda(mint).map(|(pda, _bump)| pda),
            |FeeAccount { bump, .. }| create_raw_fee_account_pda(mint, *bump),
        )
    }

    /// Yields `None` if fee account PDA could not be computed
    #[inline]
    pub fn accounts_to_update_swap<'a, I: IntoIterator<Item = &'a [u8; 32]>>(
        &self,
        mints: I,
    ) -> impl Iterator<Item = Option<[u8; 32]>> + use<'a, '_, I> {
        mints.into_iter().map(|mint| self.fee_account(mint))
    }

    #[inline]
    pub fn update_swap<'a, I: IntoIterator<Item = &'a [u8; 32]>>(
        &mut self,
        mints: I,
        fetched: &HashMap<B58PK, Account>,
    ) -> Result<(), JsError> {
        mints.into_iter().try_for_each(|mint| {
            let fee_acc = self.fee_account(mint).ok_or_else(no_valid_pda_err)?;
            let new_fee_acc = fetched
                .get(&Bs58Array(fee_acc))
                .ok_or_else(|| missing_acc_err(&fee_acc))?;
            let new_fee_acc = FeeAccountPacked::of_acc_data(&new_fee_acc.data)
                .ok_or_else(|| acc_deser_err(&fee_acc))?
                .into_fee_account();

            self.lsts.insert(*mint, new_fee_acc);

            Ok(())
        })
    }
}

fn find_fee_account_pda(lst_mint: &[u8; 32]) -> Option<([u8; 32], u8)> {
    let (s1, s2) = fee_account_seeds(lst_mint);
    find_pda(&[s1, s2], &ID)
}

fn create_raw_fee_account_pda(lst_mint: &[u8; 32], bump: u8) -> Option<[u8; 32]> {
    let (s1, s2) = fee_account_seeds(lst_mint);
    create_raw_pda(&[s1.as_slice(), s2.as_slice(), &[bump]], &ID)
}
