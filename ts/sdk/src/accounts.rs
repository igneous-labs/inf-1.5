use inf1_std::inf1_ctl_core::accounts::pool_state::PoolState as PoolStateCore;
use inf1_std::inf1_ctl_core::accounts::{
    lst_state_list::LstStatePackedList, pool_state::PoolStatePacked,
};
use inf1_std::inf1_ctl_core::typedefs::lst_state::LstState as LstStateCore;
use wasm_bindgen::prelude::*;

use crate::interface::{LstState, LstStateList, PoolState, B58PK};

/// @throws
#[wasm_bindgen(js_name = deserPoolState)]
pub fn deser_pool_state(bytes: &[u8]) -> Result<PoolState, JsError> {
    let pool_packed =
        PoolStatePacked::of_acc_data(bytes).ok_or(JsError::new("Invalid PoolState data"))?;
    let PoolStateCore {
        total_sol_value,
        admin,
        is_disabled,
        is_rebalancing,
        lp_protocol_fee_bps,
        lp_token_mint,
        pricing_program,
        protocol_fee_beneficiary,
        rebalance_authority,
        trading_protocol_fee_bps,
        version,
        padding: _,
    } = pool_packed.into_pool_state();
    Ok(PoolState {
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
    })
}

/// @throws
#[wasm_bindgen(js_name = deserLstStateList)]
pub fn deser_lst_state_list(bytes: &[u8]) -> Result<LstStateList, JsError> {
    let lst_states_packed =
        LstStatePackedList::of_acc_data(bytes).ok_or(JsError::new("Invalid LstStateList data"))?;

    let states: Vec<LstState> = lst_states_packed
        .0
        .iter()
        .map(|packed| {
            let LstStateCore {
                is_input_disabled,
                mint,
                pool_reserves_bump,
                protocol_fee_accumulator_bump,
                sol_value,
                sol_value_calculator,
                padding: _,
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
        .collect();

    Ok(LstStateList { states })
}
