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
pub fn set_pool_state(
    inf: &mut Inf,
    &PoolState {
        total_sol_value,
        trading_protocol_fee_bps,
        lp_protocol_fee_bps,
        version,
        is_disabled,
        is_rebalancing,
        admin,
        rebalance_authority,
        protocol_fee_beneficiary,
        pricing_program,
        lp_token_mint,
    }: &PoolState,
) {
    inf.0.pool = inf1_std::inf1_ctl_core::accounts::pool_state::PoolState {
        total_sol_value,
        trading_protocol_fee_bps,
        lp_protocol_fee_bps,
        version,
        is_disabled,
        is_rebalancing,
        padding: Default::default(),
        admin: admin.0,
        rebalance_authority: rebalance_authority.0,
        protocol_fee_beneficiary: protocol_fee_beneficiary.0,
        pricing_program: pricing_program.0,
        lp_token_mint: lp_token_mint.0,
    };
}

/// Returns serialized `PoolState` account data
#[wasm_bindgen(js_name = serPoolState)]
pub fn ser_pool_state(inf: &Inf) -> Box<[u8]> {
    Into::into(*inf.0.pool.as_acc_data_arr())
}

/// @throws if `pool_state_data` is invalid
#[wasm_bindgen(js_name = deserPoolState)]
pub fn deser_pool_state(inf: &mut Inf, pool_state_data: Box<[u8]>) -> Result<(), InfError> {
    inf.0.pool = inf1_std::inf1_ctl_core::accounts::pool_state::PoolStatePacked::of_acc_data(
        &pool_state_data,
    )
    .ok_or(inf1_std::err::InfErr::AccDeser { pk: POOL_STATE_ID })?
    .into_pool_state();
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
    inf.0.lst_state_list_data = lst_state_list
        .into_iter()
        .flat_map(|lst_state| {
            *inf1_std::inf1_ctl_core::typedefs::lst_state::LstState {
                is_input_disabled: lst_state.is_input_disabled,
                mint: lst_state.mint.0,
                pool_reserves_bump: lst_state.pool_reserves_bump,
                protocol_fee_accumulator_bump: lst_state.protocol_fee_accumulator_bump,
                sol_value: lst_state.sol_value,
                sol_value_calculator: lst_state.sol_value_calculator.0,
                padding: Default::default(),
            }
            .as_acc_data_arr()
        })
        .collect::<Box<[u8]>>();
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
