use std::collections::HashMap;

use inf1_std::{
    inf1_ctl_core::{
        accounts::{lst_state_list::LstStatePackedList, pool_state::PoolState},
        typedefs::lst_state::{LstState, LstStatePacked},
    },
    inf1_svc_ag_std::{SvcAg, SvcAgStd, SvcAgTy},
};
use wasm_bindgen::prelude::*;

use crate::{
    err::{acc_deser_err, missing_acc_err, missing_spl_data_err, unknown_svc_err, InfError},
    pricing::Pricing,
};

mod err;
mod init;
mod instruction;
mod interface;
mod pda;
mod pricing;
mod spl;
mod trade;
mod utils;

#[derive(Debug, Clone, PartialEq, Eq)]
#[wasm_bindgen]
pub struct Inf {
    pub(crate) pool: PoolState,
    pub(crate) lst_state_list_data: Box<[u8]>,

    /// None when mint not yet fetched
    pub(crate) lp_token_supply: Option<u64>,

    pub(crate) pricing: Pricing,

    /// key=mint
    pub(crate) lsts: HashMap<[u8; 32], (SvcAgStd, Option<Reserves>)>,

    /// [`SplPoolAccounts`].
    /// We store this in the struct so that we are able to
    /// initialize any added SPL LSTs newly added to the pool
    pub(crate) spl_lsts: HashMap<[u8; 32], [u8; 32]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Reserves {
    pub balance: u64,
}

/// Accessors
impl Inf {
    pub(crate) fn lst_state_list(&self) -> &[LstStatePacked] {
        // unwrap-safety: valid list checked at construction and update time
        LstStatePackedList::of_acc_data(&self.lst_state_list_data)
            .unwrap()
            .0
    }

    /// Lazily initializes a lst on `self.lsts`
    ///
    /// Errors if SPL data is not in `self.spl_lsts`
    /// or sol value calculator is unknown
    pub(crate) fn try_get_or_init_lst(
        &mut self,
        lst_state: &LstState,
    ) -> Result<(&mut SvcAgStd, &mut Option<Reserves>), InfError> {
        // cannot use Entry API here because that borrows self as mut,
        // so we cannot access self.lst_state_list() to init

        // need to do this contains_key() + get_mut() unwrap thing instead of matching on None
        // because otherwise self will be borrowed as mut and code below cant compile
        if self.lsts.contains_key(&lst_state.mint) {
            let (calc, reserves) = self.lsts.get_mut(&lst_state.mint).unwrap();
            return Ok((calc, reserves));
        }

        let ty = SvcAgTy::try_from_svc_program_id(&lst_state.sol_value_calculator)
            .ok_or_else(|| unknown_svc_err(&lst_state.sol_value_calculator))?;

        let init_data = match ty {
            SvcAgTy::Lido => SvcAg::Lido(()),
            SvcAgTy::Marinade => SvcAg::Marinade(()),
            SvcAgTy::SanctumSpl => {
                let mint = &lst_state.mint;
                let stake_pool_addr = self
                    .spl_lsts
                    .get(mint)
                    .ok_or_else(|| missing_spl_data_err(mint))?;
                SvcAg::SanctumSpl(*stake_pool_addr)
            }
            SvcAgTy::SanctumSplMulti => {
                let mint = &lst_state.mint;
                let stake_pool_addr = self
                    .spl_lsts
                    .get(mint)
                    .ok_or_else(|| missing_spl_data_err(mint))?;
                SvcAg::SanctumSplMulti(*stake_pool_addr)
            }
            SvcAgTy::Spl => {
                let mint = &lst_state.mint;
                let stake_pool_addr = self
                    .spl_lsts
                    .get(mint)
                    .ok_or_else(|| missing_spl_data_err(mint))?;
                SvcAg::Spl(*stake_pool_addr)
            }
            SvcAgTy::Wsol => SvcAg::Wsol(()),
        };

        let calc = SvcAgStd::new(init_data);
        let (calc, reserves) = self.lsts.entry(lst_state.mint).or_insert((calc, None));

        Ok((calc, reserves))
    }
}
