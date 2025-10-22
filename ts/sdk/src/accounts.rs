use inf1_std::inf1_ctl_core::accounts::{
    lst_state_list::LstStatePackedList, pool_state::PoolStatePacked,
};
use wasm_bindgen::prelude::*;

use crate::interface::{LstState, LstStateList, PoolState, B58PK};

/// @throws
#[wasm_bindgen(js_name = deserPoolState)]
pub fn deser_pool_state(bytes: &[u8]) -> Result<PoolState, JsError> {
    let pool_packed =
        PoolStatePacked::of_acc_data(bytes).ok_or(JsError::new("Invalid PoolState data"))?;
    let pool = pool_packed.into_pool_state();
    Ok(PoolState {
        total_sol_value: pool.total_sol_value,
        trading_protocol_fee_bps: pool.trading_protocol_fee_bps,
        lp_protocol_fee_bps: pool.lp_protocol_fee_bps,
        version: pool.version,
        is_disabled: pool.is_disabled,
        is_rebalancing: pool.is_rebalancing,
        admin: B58PK::new(pool.admin),
        rebalance_authority: B58PK::new(pool.rebalance_authority),
        protocol_fee_beneficiary: B58PK::new(pool.protocol_fee_beneficiary),
        pricing_program: B58PK::new(pool.pricing_program),
        lp_token_mint: B58PK::new(pool.lp_token_mint),
    })
}

/// @throws
#[wasm_bindgen(js_name = deserLstStateList)]
pub fn deser_lst_state_list(bytes: &[u8]) -> Result<LstStateList, JsError> {
    let lst_states_packed = LstStatePackedList::of_acc_data(bytes)
        .ok_or(JsError::new("Invalid LstStateList data"))?;

    let states: Vec<LstState> = lst_states_packed
        .0
        .iter()
        .map(|packed| {
            let lst = packed.into_lst_state();
            LstState {
                is_input_disabled: lst.is_input_disabled,
                pool_reserves_bump: lst.pool_reserves_bump,
                protocol_fee_accumulator_bump: lst.protocol_fee_accumulator_bump,
                sol_value: lst.sol_value,
                mint: B58PK::new(lst.mint),
                sol_value_calculator: B58PK::new(lst.sol_value_calculator),
            }
        })
        .collect();

    Ok(LstStateList { states })
}
