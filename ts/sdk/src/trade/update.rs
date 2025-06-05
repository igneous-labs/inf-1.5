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
    pda::controller::create_raw_pool_reserves_ata,
    trade::{Pair, PkPair},
    utils::{balance_from_token_acc_data, try_find_lst_state},
    Inf, Reserves,
};

#[wasm_bindgen(js_name = accountsToUpdateForTrade)]
pub fn accounts_to_update_for_trade(
    inf: &Inf,
    PkPair(Pair {
        inp: Bs58Array(inp),
        out: Bs58Array(out),
    }): &PkPair,
) -> Result<Box<[B58PK]>, JsError> {
    let Inf {
        pool: PoolState { lp_token_mint, .. },
        lsts,
        pricing,
        ..
    } = inf;
    let lst_state_list = inf.lst_state_list();

    let mut res = vec![B58PK::new(POOL_STATE_ID), B58PK::new(LST_STATE_LIST_ID)];

    if out == lp_token_mint {
        // add liquidity
        let (
            _i,
            LstState {
                pool_reserves_bump, ..
            },
        ) = try_find_lst_state(lst_state_list, inp)?;
        let (calc, _) = lsts.get(inp).ok_or_else(|| missing_spl_data(inp))?;
        res.extend(
            calc.accounts_to_update()
                .copied()
                .chain([
                    *lp_token_mint,
                    create_raw_pool_reserves_ata(inp, pool_reserves_bump),
                ])
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
        let [inp_res, out_res]: [Result<_, JsError>; 2] = [inp, out].map(|mint| {
            let (
                _i,
                LstState {
                    pool_reserves_bump, ..
                },
            ) = try_find_lst_state(lst_state_list, mint)?;
            let reserves = create_raw_pool_reserves_ata(mint, pool_reserves_bump);
            let (calc, _) = lsts.get(mint).ok_or_else(|| missing_spl_data(mint))?;
            Ok((calc, reserves))
        });
        let (inp_calc, inp_reserves) = inp_res?;
        let (out_calc, out_reserves) = out_res?;
        res.extend(
            inp_calc
                .accounts_to_update()
                .copied()
                .chain(out_calc.accounts_to_update().copied())
                .chain(pricing.accounts_to_update_swap([inp, out]))
                .chain([inp_reserves, out_reserves])
                .map(B58PK::new),
        );
    };

    res.sort();
    res.dedup();
    Ok(res.into_boxed_slice())
}

#[wasm_bindgen(js_name = updateForTrade)]
pub fn update_for_trade(
    inf: &mut Inf,
    PkPair(Pair {
        inp: Bs58Array(inp),
        out: Bs58Array(out),
    }): &PkPair,
    AccountMap(fetched): &AccountMap,
) -> Result<(), JsError> {
    inf.update_ctl_accounts(fetched)?;
    if *out == inf.pool.lp_token_mint {
        // add liquidity
        inf.update_lp_token_supply(fetched)?;

        let (
            _i,
            LstState {
                pool_reserves_bump, ..
            },
        ) = try_find_lst_state(inf.lst_state_list(), inp)?;
        let reserves_addr = create_raw_pool_reserves_ata(inp, pool_reserves_bump);
        let (calc, reserves) = inf.lsts.get_mut(inp).ok_or_else(|| missing_spl_data(inp))?;

        calc.update(fetched)?;

        update_reserves(reserves, reserves_addr, fetched)?;
    } else if *inp == inf.pool.lp_token_mint {
        // remove liquidity
        inf.update_lp_token_supply(fetched)?;

        let (
            _i,
            LstState {
                pool_reserves_bump, ..
            },
        ) = try_find_lst_state(inf.lst_state_list(), out)?;
        let reserves_addr = create_raw_pool_reserves_ata(out, pool_reserves_bump);
        let (calc, reserves) = inf.lsts.get_mut(out).ok_or_else(|| missing_spl_data(out))?;

        calc.update(fetched)?;

        update_reserves(reserves, reserves_addr, fetched)?;

        inf.pricing.update_remove_liquidity(fetched)?;
    } else {
        // swap
        [inp, out]
            .iter()
            .try_for_each::<_, Result<(), JsError>>(|mint| {
                let (
                    _i,
                    LstState {
                        pool_reserves_bump, ..
                    },
                ) = try_find_lst_state(inf.lst_state_list(), mint)?;
                let reserves_addr = create_raw_pool_reserves_ata(mint, pool_reserves_bump);
                let (calc, reserves) = inf
                    .lsts
                    .get_mut(*mint)
                    .ok_or_else(|| missing_spl_data(mint))?;

                calc.update(fetched)?;

                update_reserves(reserves, reserves_addr, fetched)?;

                Ok(())
            })?;

        inf.pricing.update_swap([inp, out], fetched)?;
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
