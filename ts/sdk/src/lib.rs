use std::collections::HashMap;

use inf1_core::inf1_ctl_core::{
    accounts::{
        lst_state_list::LstStatePackedList,
        pool_state::{PoolState, PoolStatePacked},
    },
    keys::{LST_STATE_LIST_ID, POOL_STATE_ID},
};
use serde::{Deserialize, Serialize};
use tsify_next::Tsify;
use wasm_bindgen::prelude::*;

use crate::{
    err::{acc_deser_err, missing_acc_err},
    interface::{Account, SplPoolAccounts, B58PK},
    pricing::FlatFeePricing,
    sol_val_calc::Calc,
    utils::token_supply_from_mint_data,
};

mod err;
mod interface;
mod pda;
mod pricing;
mod sol_val_calc;
mod swap;
mod utils;

#[derive(Debug, Clone, PartialEq, Eq)]
#[wasm_bindgen]
pub struct InfHandle {
    pub(crate) pool: PoolState,
    pub(crate) lst_state_list_data: Box<[u8]>,
    pub(crate) lp_token_supply: Option<u64>,

    pub(crate) pricing: FlatFeePricing,

    /// key=mint
    pub(crate) sol_val_calcs: HashMap<[u8; 32], Calc>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi, large_number_types_as_bigints)]
#[serde(rename_all = "camelCase")]
pub struct InitAccounts {
    pub pool_state: Account,
    pub lst_state_list: Account,
}

#[wasm_bindgen(js_name = initInf)]
pub fn inf_init(
    InitAccounts {
        pool_state,
        lst_state_list,
    }: InitAccounts,
    SplPoolAccounts(spl_lsts): &SplPoolAccounts,
) -> Result<InfHandle, JsError> {
    let pool = PoolStatePacked::of_acc_data(&pool_state.data)
        .ok_or_else(|| acc_deser_err(&POOL_STATE_ID))?
        .into_pool_state();
    let lst_state_list_packed = LstStatePackedList::of_acc_data(&lst_state_list.data)
        .ok_or_else(|| acc_deser_err(&LST_STATE_LIST_ID))?;

    let sol_val_calcs: Result<HashMap<[u8; 32], Calc>, JsError> = lst_state_list_packed
        .0
        .iter()
        .map(|s| {
            let s = s.into_lst_state();
            Ok((s.mint, Calc::new(&s, spl_lsts)?))
        })
        .collect();

    Ok(InfHandle {
        pool,
        lst_state_list_data: lst_state_list.data,
        lp_token_supply: None,
        pricing: FlatFeePricing::default(),
        sol_val_calcs: sol_val_calcs?,
    })
}

impl InfHandle {
    pub(crate) fn update_ctl_accounts(
        &mut self,
        fetched: &HashMap<B58PK, Account>,
    ) -> Result<(), JsError> {
        let [pool_state_acc, lst_state_list_acc] = [POOL_STATE_ID, LST_STATE_LIST_ID].map(|pk| {
            fetched
                .get(&B58PK::new(pk))
                .ok_or_else(|| missing_acc_err(&pk))
        });
        let pool_state_acc = pool_state_acc?;
        let lst_state_list_acc = lst_state_list_acc?;

        let pool = PoolStatePacked::of_acc_data(&pool_state_acc.data)
            .ok_or_else(|| acc_deser_err(&POOL_STATE_ID))?;
        LstStatePackedList::of_acc_data(&lst_state_list_acc.data)
            .ok_or_else(|| acc_deser_err(&LST_STATE_LIST_ID))?;

        self.pool = pool.into_pool_state();
        self.lst_state_list_data = lst_state_list_acc.data.clone();

        Ok(())
    }

    pub(crate) fn update_lp_token_supply(
        &mut self,
        fetched: &HashMap<B58PK, Account>,
    ) -> Result<(), JsError> {
        let lp_mint_acc = fetched
            .get(&B58PK::new(self.pool.lp_token_mint))
            .ok_or_else(|| missing_acc_err(&self.pool.lp_token_mint))?;
        let lp_token_supply = token_supply_from_mint_data(&lp_mint_acc.data)
            .ok_or_else(|| acc_deser_err(&self.pool.lp_token_mint))?;

        self.lp_token_supply = Some(lp_token_supply);

        Ok(())
    }
}
