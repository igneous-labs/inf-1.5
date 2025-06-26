use bs58_fixed_wasm::Bs58Array;
use inf1_core::inf1_ctl_core::{
    accounts::pool_state::PoolStatePacked,
    keys::{LST_STATE_LIST_ID, POOL_STATE_ID},
};
use wasm_bindgen::prelude::*;

use crate::{
    err::{acc_deser_err, missing_acc_err, InfError},
    interface::{AccountMap, SplPoolAccounts, B58PK},
    Inf,
};

/// Returns the pubkeys of the accounts that need ot be fetched to initialize
/// a new {@link Inf} object
#[wasm_bindgen(js_name = initPks)]
pub fn init_pks() -> Box<[B58PK]> {
    [POOL_STATE_ID, LST_STATE_LIST_ID].map(B58PK::new).into()
}

/// Initialize a new {@link Inf} object.
///
/// The returned object must be updated for a mint pair before it is ready to
/// quote and operate for trades involving that pair
///
/// @throws
#[wasm_bindgen(js_name = init)]
pub fn init(
    AccountMap(mut fetched): AccountMap,
    SplPoolAccounts(spl_lsts): SplPoolAccounts,
) -> Result<Inf, InfError> {
    let [p, l] = [POOL_STATE_ID, LST_STATE_LIST_ID].map(|pk| {
        fetched
            .remove(&B58PK::new(pk))
            .ok_or_else(|| missing_acc_err(&pk))
    });
    let pool_state = p?;
    let lst_state_list = l?;

    let pool = PoolStatePacked::of_acc_data(&pool_state.data)
        .ok_or_else(|| acc_deser_err(&POOL_STATE_ID))?
        .into_pool_state();
    let lst_state_list_data = lst_state_list.data.into_vec().into_boxed_slice();
    let spl_lsts = spl_lsts
        .into_iter()
        .map(|(Bs58Array(k), Bs58Array(v))| (k, v))
        .collect();

    Ok(Inf {
        pool,
        lst_state_list_data,
        spl_lsts,
        lp_token_supply: None,
        pricing: Default::default(),
        lsts: Default::default(),
    })
}
