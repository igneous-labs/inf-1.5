use std::collections::HashMap;

use bs58_fixed_wasm::Bs58Array;
use inf1_pp_flatfee_core::{
    accounts::{
        fee::{FeeAccount, FeeAccountPacked},
        program_state::{ProgramState, ProgramStatePacked},
    },
    instructions::pricing::{
        lp::{mint::FlatFeeMintLpAccs, redeem::FlatFeeRedeemLpAccs},
        price::{FlatFeePriceAccs, NewIxSufAccsBuilder},
    },
    pricing::{
        lp::{FlatFeeMintLpPricing, FlatFeeRedeemLpPricing},
        price::FlatFeeSwapPricing,
    },
};

use crate::{
    err::{acc_deser_err, missing_acc_err, InfError},
    interface::{Account, B58PK},
    pda::pricing::{create_raw_fee_account_pda, find_fee_account_pda},
    trade::Pair,
};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct FlatFeePricing {
    /// `None` when acc not yet fetched
    pub lp_withdrawal_fee_bps: Option<u16>,

    /// key=mint
    ///
    /// Entry does not exist if acc not yet fetched
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
    ) -> Result<(), InfError> {
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
    pub fn fee_account(&self, mint: &[u8; 32]) -> [u8; 32] {
        self.lsts.get(mint).map_or_else(
            // unwrap-safety: fee accounts should all be of valid, found PDAs
            || find_fee_account_pda(mint).unwrap().0,
            |FeeAccount { bump, .. }| create_raw_fee_account_pda(mint, *bump),
        )
    }

    /// Yields `None` if fee account PDA could not be computed
    #[inline]
    pub fn accounts_to_update_swap<'a, I: IntoIterator<Item = &'a [u8; 32]>>(
        &self,
        mints: I,
    ) -> impl Iterator<Item = [u8; 32]> + use<'a, '_, I> {
        mints.into_iter().map(|mint| self.fee_account(mint))
    }

    #[inline]
    pub fn update_swap<'a, I: IntoIterator<Item = &'a [u8; 32]>>(
        &mut self,
        mints: I,
        fetched: &HashMap<B58PK, Account>,
    ) -> Result<(), InfError> {
        mints.into_iter().try_for_each(|mint| {
            let fee_acc = self.fee_account(mint);
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

/// Pricing traits
impl FlatFeePricing {
    pub(crate) const fn to_price_lp_tokens_to_mint(&self) -> FlatFeeMintLpPricing {
        FlatFeeMintLpPricing
    }

    pub(crate) const fn to_price_lp_tokens_to_mint_accs(&self) -> FlatFeeMintLpAccs {
        FlatFeeMintLpAccs
    }

    pub(crate) const fn to_price_lp_tokens_to_redeem(&self) -> Option<FlatFeeRedeemLpPricing> {
        match self.lp_withdrawal_fee_bps {
            None => None,
            Some(lp_withdrawal_fee_bps) => Some(FlatFeeRedeemLpPricing {
                lp_withdrawal_fee_bps,
            }),
        }
    }

    pub(crate) const fn to_price_lp_tokens_to_redeem_accs(&self) -> FlatFeeRedeemLpAccs {
        FlatFeeRedeemLpAccs::MAINNET
    }

    pub(crate) fn to_price_swap(
        &self,
        Pair { inp, out }: &Pair<&[u8; 32]>,
    ) -> Option<FlatFeeSwapPricing> {
        let [Some(FeeAccount { input_fee_bps, .. }), Some(FeeAccount { output_fee_bps, .. })] =
            [inp, out].map(|mint| self.lsts.get(*mint))
        else {
            return None;
        };
        Some(FlatFeeSwapPricing {
            input_fee_bps: *input_fee_bps,
            output_fee_bps: *output_fee_bps,
        })
    }

    pub(crate) fn to_price_swap_accs(
        &self,
        Pair { inp, out }: &Pair<&[u8; 32]>,
    ) -> FlatFeePriceAccs {
        FlatFeePriceAccs(
            NewIxSufAccsBuilder::start()
                .with_input_fee(self.fee_account(inp))
                .with_output_fee(self.fee_account(out))
                .build(),
        )
    }
}
