use bs58_fixed_wasm::Bs58Array;
use inf1_core::inf1_ctl_core::{
    accounts::{lst_state_list::LstStatePackedList, pool_state::PoolStatePacked},
    keys::{LST_STATE_LIST_ID, POOL_STATE_ID},
};
use serde::{Deserialize, Serialize};
use tsify_next::Tsify;
use wasm_bindgen::prelude::*;

use crate::{
    err::acc_deser_err,
    interface::{Account, SplPoolAccounts, B58PK},
    pricing::FlatFeePricing,
    sol_val_calc::Calc,
    Inf,
};

#[derive(Debug, Default, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi, large_number_types_as_bigints)]
#[serde(rename_all = "camelCase")]
pub struct Init<T> {
    pub pool_state: T,
    pub lst_state_list: T,
}

// need to use a simple newtype here instead of type alias
// otherwise wasm_bindgen shits itself with missing generics
#[derive(Debug, Default, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi, large_number_types_as_bigints)]
#[serde(rename_all = "camelCase")]
pub struct InitPks(Init<B58PK>);

#[wasm_bindgen(js_name = initPks)]
pub fn init_pks() -> InitPks {
    InitPks(Init {
        pool_state: B58PK::new(POOL_STATE_ID),
        lst_state_list: B58PK::new(LST_STATE_LIST_ID),
    })
}

// need to use a simple newtype here instead of type alias
// otherwise wasm_bindgen shits itself with missing generics
#[derive(Debug, Default, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi, large_number_types_as_bigints)]
#[serde(rename_all = "camelCase")]
pub struct InitAccounts(Init<Account>);

#[wasm_bindgen(js_name = init)]
pub fn init(
    InitAccounts(Init {
        pool_state,
        lst_state_list,
    }): InitAccounts,
    SplPoolAccounts(spl_lsts): SplPoolAccounts,
) -> Result<Inf, JsError> {
    let pool = PoolStatePacked::of_acc_data(&pool_state.data)
        .ok_or_else(|| acc_deser_err(&POOL_STATE_ID))?
        .into_pool_state();
    let lst_state_list_packed = LstStatePackedList::of_acc_data(&lst_state_list.data)
        .ok_or_else(|| acc_deser_err(&LST_STATE_LIST_ID))?;
    let spl_lsts = spl_lsts
        .into_iter()
        .map(|(Bs58Array(k), Bs58Array(v))| (k, v))
        .collect();

    let lsts: Result<_, JsError> = lst_state_list_packed
        .0
        .iter()
        .map(|s| {
            let s = s.into_lst_state();
            Ok((s.mint, (Calc::new(&s, &spl_lsts)?, None)))
        })
        .collect();

    Ok(Inf {
        pool,
        lst_state_list_data: lst_state_list.data.into_vec().into_boxed_slice(),
        lp_token_supply: None,
        pricing: FlatFeePricing::default(),
        lsts: lsts?,
        spl_lsts,
    })
}
