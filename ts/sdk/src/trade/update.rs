use std::collections::HashMap;

use bs58_fixed_wasm::Bs58Array;
use inf1_core::inf1_ctl_core::{
    accounts::pool_state::PoolState,
    keys::{LST_STATE_LIST_ID, POOL_STATE_ID},
    typedefs::lst_state::LstState,
};
use wasm_bindgen::prelude::*;

use crate::{
    acc_deser_err,
    err::missing_spl_data,
    interface::{Account, AccountMap, B58PK},
    missing_acc_err,
    trade::PkPair,
    utils::{balance_from_token_acc_data, create_raw_pool_reserves_ata, try_find_lst_state},
    InfHandle, Reserves,
};

#[wasm_bindgen(js_name = accountsToUpdateForSwap)]
pub fn accounts_to_update_for_swap(
    inf: &InfHandle,
    PkPair {
        inp: Bs58Array(inp),
        out: Bs58Array(out),
    }: &PkPair,
) -> Result<Box<[B58PK]>, JsError> {
    let InfHandle {
        pool: PoolState { lp_token_mint, .. },
        lsts,
        pricing,
        ..
    } = inf;
    let lst_state_list = inf.lst_state_list();

    let mut res = vec![B58PK::new(POOL_STATE_ID), B58PK::new(LST_STATE_LIST_ID)];

    if out == lp_token_mint {
        // add liquidity
        let (calc, _) = lsts.get(inp).ok_or_else(|| missing_spl_data(inp))?;
        res.extend(
            calc.accounts_to_update()
                .copied()
                .chain([*lp_token_mint])
                .map(B58PK::new),
        );
    } else if inp == lp_token_mint {
        // remove liquidity
        let (
            _i,
            LstState {
                pool_reserves_bump, ..
            },
        ) = try_find_lst_state(lst_state_list, out)?;
        let (calc, _) = lsts.get(out).ok_or_else(|| missing_spl_data(out))?;
        res.extend(
            calc.accounts_to_update()
                .copied()
                .chain([
                    *lp_token_mint,
                    pricing.account_to_update_remove_liquidity(),
                    create_raw_pool_reserves_ata(out, pool_reserves_bump),
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

#[wasm_bindgen(js_name = updateForSwap)]
pub fn update_for_swap(
    inf: &mut InfHandle,
    PkPair {
        inp: Bs58Array(inp),
        out: Bs58Array(out),
    }: &PkPair,
    AccountMap(fetched): &AccountMap,
) -> Result<(), JsError> {
    inf.update_ctl_accounts(fetched)?;
    if *out == inf.pool.lp_token_mint {
        // add liquidity
        inf.update_lp_token_supply(fetched)?;

        let (calc, _) = inf.lsts.get_mut(inp).ok_or_else(|| missing_spl_data(inp))?;

        calc.update(fetched)?;
    } else if *inp == inf.pool.lp_token_mint {
        // remove liquidity
        inf.update_lp_token_supply(fetched)?;

        let (
            _i,
            LstState {
                pool_reserves_bump, ..
            },
        ) = try_find_lst_state(inf.lst_state_list(), out)?;
        let (calc, reserves) = inf.lsts.get_mut(out).ok_or_else(|| missing_spl_data(out))?;
        let reserves_addr = create_raw_pool_reserves_ata(out, pool_reserves_bump);

        calc.update(fetched)?;

        update_reserves(reserves, reserves_addr, fetched)?;

        inf.pricing.update_remove_liquidity(fetched)?;
    } else {
        // swap
        todo!()
    };

    Ok(())
}

fn update_reserves(
    reserves: &mut Option<Reserves>,
    reserves_addr: [u8; 32],
    fetched: &HashMap<B58PK, Account>,
) -> Result<(), JsError> {
    let token_acc = fetched
        .get(&B58PK::new(reserves_addr))
        .ok_or_else(|| missing_acc_err(&reserves_addr))?;
    *reserves = Some(Reserves {
        balance: balance_from_token_acc_data(&token_acc.data)
            .ok_or_else(|| acc_deser_err(&reserves_addr))?,
    });
    Ok(())
}
