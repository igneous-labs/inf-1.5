use std::iter::once;

use bs58_fixed_wasm::Bs58Array;
use inf1_core::inf1_ctl_core::keys::{LST_STATE_LIST_ID, POOL_STATE_ID};
use serde::{Deserialize, Serialize};
use tsify_next::Tsify;
use wasm_bindgen::prelude::*;

use crate::{
    err::missing_spl_data,
    interface::{AccountMap, B58PK},
    InfHandle,
};

#[derive(Debug, Default, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct SwapMints {
    pub inp: B58PK,
    pub out: B58PK,
}

#[wasm_bindgen(js_name = infAccountsToUpdateForSwap)]
pub fn inf_accounts_to_update_for_swap(
    inf: &InfHandle,
    SwapMints {
        inp: Bs58Array(inp),
        out: Bs58Array(out),
    }: &SwapMints,
) -> Result<Box<[B58PK]>, JsError> {
    let mut res = vec![B58PK::new(POOL_STATE_ID), B58PK::new(LST_STATE_LIST_ID)];

    if *out == inf.pool.lp_token_mint {
        // add liquidity
        let calc = inf
            .sol_val_calcs
            .get(inp)
            .ok_or_else(|| missing_spl_data(inp))?;
        res.extend(
            calc.accounts_to_update()
                .copied()
                .chain(once(inf.pool.lp_token_mint))
                .map(B58PK::new),
        );
    } else if *inp == inf.pool.lp_token_mint {
        // remove liquidity
        let calc = inf
            .sol_val_calcs
            .get(out)
            .ok_or_else(|| missing_spl_data(out))?;
        res.extend(
            calc.accounts_to_update()
                .copied()
                .chain([
                    inf.pool.lp_token_mint,
                    inf.pricing.account_to_update_remove_liquidity(),
                ])
                .map(B58PK::new),
        );
    } else {
        // swap
        todo!()
    };

    res.sort();
    res.dedup();
    Ok(res.into_boxed_slice())
}

#[wasm_bindgen(js_name = infUpdateForSwap)]
pub fn inf_update_for_swap(
    inf: &mut InfHandle,
    SwapMints {
        inp: Bs58Array(inp),
        out: Bs58Array(out),
    }: &SwapMints,
    AccountMap(fetched): &AccountMap,
) -> Result<(), JsError> {
    inf.update_ctl_accounts(fetched)?;
    if *out == inf.pool.lp_token_mint {
        // add liquidity
        inf.update_lp_token_supply(fetched)?;
        let calc = inf
            .sol_val_calcs
            .get_mut(inp)
            .ok_or_else(|| missing_spl_data(inp))?;
        calc.update(fetched)?;
    } else if *inp == inf.pool.lp_token_mint {
        // remove liquidity
        inf.update_lp_token_supply(fetched)?;
        let calc = inf
            .sol_val_calcs
            .get_mut(out)
            .ok_or_else(|| missing_spl_data(out))?;
        calc.update(fetched)?;
        inf.pricing.update_remove_liquidity(fetched)?;
    } else {
        // swap
        todo!()
    };

    Ok(())
}
