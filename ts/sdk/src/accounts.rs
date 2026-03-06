use inf1_std::inf1_ctl_core::{
    accounts::{lst_state_list::LstStatePackedList, pool_state::VerPoolState},
    keys::{LST_STATE_LIST_ID, POOL_STATE_ID},
    typedefs::versioned::V1_2,
};
use wasm_bindgen::prelude::*;

use crate::{
    err::InfError,
    interface::{
        lst_state_from_intf, lst_state_into_intf, pool_state_v2_from_intf, pool_state_v2_into_intf,
        LstState, PoolStateV2,
    },
    Inf,
};

#[wasm_bindgen(js_name = getPoolState)]
pub fn get_pool_state(inf: &Inf) -> PoolStateV2 {
    let ps = inf.0.pool.migrated(0);
    pool_state_v2_into_intf(ps)
}

/// Sets the `PoolState` account data
#[wasm_bindgen(js_name = setPoolState)]
pub fn set_pool_state(inf: &mut Inf, intf: &PoolStateV2) {
    inf.0.pool = V1_2::V2(pool_state_v2_from_intf(*intf));
}

/// Returns serialized `PoolState` account data
#[wasm_bindgen(js_name = serPoolState)]
pub fn ser_pool_state(inf: &Inf) -> Box<[u8]> {
    Into::into(inf.0.pool.as_acc_data_arr())
}

/// @throws if `pool_state_data` is invalid
#[wasm_bindgen(js_name = deserPoolState)]
pub fn deser_pool_state(inf: &mut Inf, pool_state_data: Box<[u8]>) -> Result<(), InfError> {
    inf.0.pool = VerPoolState::try_from_acc_data(&pool_state_data)
        .ok_or(inf1_std::err::InfErr::AccDeser { pk: POOL_STATE_ID })?;
    Ok(())
}

/// @throws if stored lst state list account data is invalid
#[wasm_bindgen(js_name = getLstStateList)]
pub fn get_lst_state_list(inf: &Inf) -> Result<Vec<LstState>, InfError> {
    Ok(inf
        .0
        .try_lst_state_list()?
        .iter()
        .map(|packed| lst_state_into_intf(packed.into_lst_state()))
        .collect())
}

/// Sets the `LstStateList` account data
#[wasm_bindgen(js_name = setLstStateList)]
pub fn set_lst_state_list(
    inf: &mut Inf,
    // Clippy complains, needed for wasm_bindgen
    #[allow(clippy::boxed_local)] lst_state_list: Box<[LstState]>,
) {
    inf.0.lst_state_list_data = lst_state_list
        .iter()
        .flat_map(|intf| *lst_state_from_intf(*intf).as_acc_data_arr())
        .collect();
}

/// Returns serialized `LstStateList` account data
#[wasm_bindgen(js_name = serLstStateList)]
pub fn ser_lst_state_list(inf: &Inf) -> Box<[u8]> {
    inf.0.lst_state_list_data.clone()
}

/// @throws if `lst_state_list_data` is invalid
#[wasm_bindgen(js_name = deserLstStateList)]
pub fn deser_lst_state_list(inf: &mut Inf, lst_state_list_data: Box<[u8]>) -> Result<(), InfError> {
    LstStatePackedList::of_acc_data(&lst_state_list_data).ok_or(
        inf1_std::err::InfErr::AccDeser {
            pk: LST_STATE_LIST_ID,
        },
    )?;

    inf.0.lst_state_list_data = lst_state_list_data;

    Ok(())
}
