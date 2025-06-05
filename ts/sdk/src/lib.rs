use std::collections::{hash_map::Entry, HashMap};

use bs58_fixed_wasm::Bs58Array;
use inf1_core::inf1_ctl_core::{
    accounts::{
        lst_state_list::LstStatePackedList,
        packed_list::PackedList,
        pool_state::{PoolState, PoolStatePacked},
    },
    keys::{LST_STATE_LIST_ID, POOL_STATE_ID},
    typedefs::lst_state::LstStatePacked,
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
mod instruction;
mod interface;
mod pda;
mod pricing;
mod sol_val_calc;
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
        lst_state_list_data: lst_state_list.data,
        lp_token_supply: None,
        pricing: FlatFeePricing::default(),
        lsts: lsts?,
        spl_lsts,
    })
}

/// Update SPL LSTs auxiliary data to support new SPL LSTs that may have previously not been covered
#[wasm_bindgen(js_name = updateSplLsts)]
pub fn update_spl_lsts(inf: &mut Inf, SplPoolAccounts(spl_lsts): SplPoolAccounts) {
    inf.spl_lsts = spl_lsts
        .into_iter()
        .map(|(Bs58Array(k), Bs58Array(v))| (k, v))
        .collect();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Reserves {
    pub balance: u64,
}

/// Update
impl Inf {
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
        let PackedList(lst_state_list) = LstStatePackedList::of_acc_data(&lst_state_list_acc.data)
            .ok_or_else(|| acc_deser_err(&LST_STATE_LIST_ID))?;
        lst_state_list.iter().for_each(|s| {
            let s = s.into_lst_state();
            // Initialize sol value calc and indiv LST data if newly added LST
            if let Entry::Vacant(entry) = self.lsts.entry(s.mint) {
                // TODO: we are ignoring Calc::new() error here
                // so that we dont brick our stuff from adding a new unsupported SPL LST.
                // We maybe want to handle this error properly instead
                if let Ok(calc) = Calc::new(&s, &self.spl_lsts) {
                    entry.insert((calc, None));
                }
            }
        });
        // TODO: maybe cleanup removed LSTs from self.lsts?

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

/// Accessors
impl Inf {
    pub(crate) fn lst_state_list(&self) -> &[LstStatePacked] {
        // unwrap-safety: valid list checked at construction and update time
        LstStatePackedList::of_acc_data(&self.lst_state_list_data)
            .unwrap()
            .0
    }
}
