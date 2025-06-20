use std::collections::HashMap;

use inf1_core::inf1_ctl_core::{
    accounts::{lst_state_list::LstStatePackedList, pool_state::PoolState},
    typedefs::lst_state::LstStatePacked,
};
use wasm_bindgen::prelude::*;

use crate::{
    err::{acc_deser_err, missing_acc_err},
    pricing::FlatFeePricing,
    sol_val_calc::Calc,
};

mod err;
mod init;
mod instruction;
mod interface;
mod pda;
mod pricing;
mod sol_val_calc;
mod spl;
mod trade;
mod utils;

#[derive(Debug, Clone, PartialEq, Eq)]
#[wasm_bindgen]
pub struct Inf {
    pub(crate) pool: PoolState,
    pub(crate) lst_state_list_data: Box<[u8]>,
    pub(crate) lp_token_supply: Option<u64>,

    pub(crate) pricing: FlatFeePricing,

    /// key=mint
    pub(crate) lsts: HashMap<[u8; 32], (Calc, Option<Reserves>)>,

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
}
