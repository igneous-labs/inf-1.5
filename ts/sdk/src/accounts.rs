use inf1_std::inf1_ctl_core::{
    accounts::lst_state_list::LstStatePackedList,
    keys::{LST_STATE_LIST_ID, POOL_STATE_ID},
};
use wasm_bindgen::prelude::*;

use crate::{
    err::InfError,
    interface::{LstState, PoolState, B58PK},
    Inf,
};

#[wasm_bindgen(js_name = getPoolState)]
pub fn get_pool_state(inf: &Inf) -> PoolState {
    let inf1_std::inf1_ctl_core::accounts::pool_state::PoolState {
        total_sol_value,
        admin,
        is_disabled,
        is_rebalancing,
        lp_protocol_fee_bps,
        lp_token_mint,
        padding: _,
        pricing_program,
        protocol_fee_beneficiary,
        rebalance_authority,
        trading_protocol_fee_bps,
        version,
    } = inf.0.pool;
    PoolState {
        total_sol_value,
        admin: B58PK::new(admin),
        is_disabled,
        is_rebalancing,
        lp_protocol_fee_bps,
        lp_token_mint: B58PK::new(lp_token_mint),
        pricing_program: B58PK::new(pricing_program),
        protocol_fee_beneficiary: B58PK::new(protocol_fee_beneficiary),
        rebalance_authority: B58PK::new(rebalance_authority),
        trading_protocol_fee_bps,
        version,
    }
}

/// Sets the `PoolState` account data
#[wasm_bindgen(js_name = setPoolState)]
pub fn set_pool_state(inf: &mut Inf, pool_state: &PoolState) {
    let packed = inf1_std::inf1_ctl_core::accounts::pool_state::PoolState {
        total_sol_value: pool_state.total_sol_value,
        trading_protocol_fee_bps: pool_state.trading_protocol_fee_bps,
        lp_protocol_fee_bps: pool_state.lp_protocol_fee_bps,
        version: pool_state.version,
        is_disabled: pool_state.is_disabled,
        is_rebalancing: pool_state.is_rebalancing,
        padding: inf.0.pool.padding,
        admin: pool_state.admin.0,
        rebalance_authority: pool_state.rebalance_authority.0,
        protocol_fee_beneficiary: pool_state.protocol_fee_beneficiary.0,
        pricing_program: pool_state.pricing_program.0,
        lp_token_mint: pool_state.lp_token_mint.0,
    };

    inf.0.pool = packed;
}

/// Returns serialized `PoolState` account data
#[wasm_bindgen(js_name = serPoolState)]
pub fn ser_pool_state(inf: &Inf) -> Box<[u8]> {
    Into::into(*inf.0.pool.as_acc_data_arr())
}

/// @throws if `pool_state_data` is invalid
#[wasm_bindgen(js_name = deserPoolState)]
pub fn deser_pool_state(inf: &mut Inf, pool_state_data: Vec<u8>) -> Result<(), InfError> {
    use inf1_std::inf1_ctl_core::accounts::pool_state::PoolState;

    if pool_state_data.len() != std::mem::size_of::<PoolState>() {
        return Err(Into::into(inf1_std::err::InfErr::AccDeser {
            pk: POOL_STATE_ID,
        }));
    }

    // safety:
    // - PoolState is POD
    // - length is validated
    unsafe {
        std::ptr::copy_nonoverlapping(
            pool_state_data.as_ptr(),
            &mut inf.0.pool as *mut PoolState as *mut u8,
            size_of::<PoolState>(),
        );
    }

    Ok(())
}

/// @throws if stored lst state list account data is invalid
#[wasm_bindgen(js_name = getLstStateList)]
pub fn get_lst_state_list(inf: &Inf) -> Result<Vec<LstState>, InfError> {
    Ok(inf
        .0
        .try_lst_state_list()?
        .iter()
        .map(|packed| {
            let inf1_std::inf1_ctl_core::typedefs::lst_state::LstState {
                is_input_disabled,
                mint,
                padding: _,
                pool_reserves_bump,
                protocol_fee_accumulator_bump,
                sol_value,
                sol_value_calculator,
            } = packed.into_lst_state();
            LstState {
                is_input_disabled,
                mint: B58PK::new(mint),
                pool_reserves_bump,
                protocol_fee_accumulator_bump,
                sol_value,
                sol_value_calculator: B58PK::new(sol_value_calculator),
            }
        })
        .collect())
}

/// Sets the `LstStateList` account data
#[wasm_bindgen(js_name = setLstStateList)]
pub fn set_lst_state_list(inf: &mut Inf, lst_state_list: Vec<LstState>) {
    use inf1_std::inf1_ctl_core::typedefs::lst_state::LstStatePacked;
    let lst_state_list = lst_state_list
        .iter()
        .map(|lst_state| {
            LstStatePacked::new(
                lst_state.is_input_disabled,
                lst_state.pool_reserves_bump,
                lst_state.protocol_fee_accumulator_bump,
                lst_state.sol_value,
                lst_state.mint.0,
                lst_state.sol_value_calculator.0,
            )
        })
        .collect::<Vec<LstStatePacked>>();

    let len_bytes = lst_state_list.len() * std::mem::size_of::<LstStatePacked>();
    let ptr = Box::into_raw(lst_state_list.into_boxed_slice()) as *mut u8;
    let lst_state_list_data =
        unsafe { Box::from_raw(std::slice::from_raw_parts_mut(ptr, len_bytes)) };
    inf.0.lst_state_list_data = lst_state_list_data;
}

/// Returns serialized `LstStateList` account data
#[wasm_bindgen(js_name = serLstStateList)]
pub fn ser_lst_state_list(inf: &Inf) -> Box<[u8]> {
    inf.0.lst_state_list_data.clone()
}

/// @throws if `lst_state_list_data` is invalid
#[wasm_bindgen(js_name = deserLstStateList)]
pub fn deser_lst_state_list(inf: &mut Inf, lst_state_list_data: Vec<u8>) -> Result<(), InfError> {
    if let None = LstStatePackedList::of_acc_data(&lst_state_list_data) {
        return Err(Into::into(inf1_std::err::InfErr::AccDeser {
            pk: LST_STATE_LIST_ID,
        }));
    }

    let len_bytes = lst_state_list_data.len();
    let ptr = Box::into_raw(lst_state_list_data.into_boxed_slice()) as *mut u8;
    let lst_state_list_data =
        unsafe { Box::from_raw(std::slice::from_raw_parts_mut(ptr, len_bytes)) };
    inf.0.lst_state_list_data = lst_state_list_data;

    Ok(())
}
