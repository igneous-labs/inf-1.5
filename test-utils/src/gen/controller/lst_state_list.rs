use std::{
    collections::{HashMap, HashSet},
    ops::RangeInclusive,
};

use generic_array_struct::generic_array_struct;
use inf1_ctl_core::typedefs::lst_state::LstState;
use inf1_svc_lido_core::solido_legacy_core::TOKENKEG_PROGRAM;
use proptest::{collection::vec, prelude::*};

use crate::{
    bool_strat, bool_to_u8, find_pool_reserves, find_protocol_fee_accumulator, pk_strat, u64_strat,
};

#[generic_array_struct(builder pub)]
#[derive(Debug, Clone, Copy, Default)]
pub struct LstStatePks<T> {
    pub mint: T,
    pub sol_value_calculator: T,
}

#[derive(Debug, Clone, Copy)]
pub struct LstStateData {
    pub lst_state: LstState,
    pub protocol_fee_accumulator: [u8; 32],
    pub pool_reserves: [u8; 32],
}

pub fn gen_lst_state(
    is_input_disabled: bool,
    sol_value: u64,
    pks: LstStatePks<[u8; 32]>,
) -> LstStateData {
    let (protocol_fee_accumulator, protocol_fee_accumulator_bump) =
        find_protocol_fee_accumulator(&TOKENKEG_PROGRAM, pks.mint());
    let (pool_reserves, pool_reserves_bump) = find_pool_reserves(&TOKENKEG_PROGRAM, pks.mint());
    LstStateData {
        lst_state: LstState {
            is_input_disabled: bool_to_u8(is_input_disabled),
            pool_reserves_bump,
            protocol_fee_accumulator_bump,
            padding: [0u8; 5],
            sol_value,
            mint: *pks.mint(),
            sol_value_calculator: *pks.sol_value_calculator(),
        },
        protocol_fee_accumulator: protocol_fee_accumulator.to_bytes(),
        pool_reserves: pool_reserves.to_bytes(),
    }
}

/// If `Option::None`, `any()` is used
#[derive(Debug, Clone, Default)]
pub struct GenLstStateArgs {
    pub is_input_disabled: Option<BoxedStrategy<bool>>,
    pub sol_value: Option<BoxedStrategy<u64>>,
    pub pks: LstStatePks<Option<BoxedStrategy<[u8; 32]>>>,
}

pub fn any_lst_state(
    GenLstStateArgs {
        is_input_disabled,
        sol_value,
        pks,
    }: GenLstStateArgs,
) -> impl Strategy<Value = LstStateData> {
    let is_input_disabled = bool_strat(is_input_disabled);
    let sol_value = u64_strat(sol_value);
    let pks = pks.0.map(pk_strat);
    (is_input_disabled, sol_value, pks).prop_map(|(is_input_disabled, sol_value, pks)| {
        gen_lst_state(is_input_disabled, sol_value, LstStatePks(pks))
    })
}

#[derive(Debug, Clone)]
pub struct LstStateListData {
    pub lst_state_list: Vec<u8>,

    /// Map `mint: token acc`
    pub protocol_fee_accumulators: HashMap<[u8; 32], [u8; 32]>,

    /// Map `mint: token acc`
    pub all_pool_reserves: HashMap<[u8; 32], [u8; 32]>,
}

pub fn any_lst_state_list(
    args: GenLstStateArgs,
    len: RangeInclusive<usize>,
) -> impl Strategy<Value = LstStateListData> {
    vec(any_lst_state(args), len).prop_map(|v| {
        let mut acc_data = Vec::new();
        let mut protocol_fee_accumulators = HashMap::new();
        let mut all_pool_reserves = HashMap::new();
        let mut dedup_mints = HashSet::new();

        v.into_iter().for_each(
            |LstStateData {
                 lst_state,
                 protocol_fee_accumulator,
                 pool_reserves,
             }| {
                if !dedup_mints.insert(lst_state.mint) {
                    return;
                }
                acc_data.extend(lst_state.as_acc_data_arr());
                protocol_fee_accumulators.insert(lst_state.mint, protocol_fee_accumulator);
                all_pool_reserves.insert(lst_state.mint, pool_reserves);
            },
        );

        LstStateListData {
            lst_state_list: acc_data,
            protocol_fee_accumulators,
            all_pool_reserves,
        }
    })
}
