use std::{collections::HashMap, iter};

use bs58_fixed_wasm::Bs58Array;
use inf1_pp_ag_std::update::{
    mint_lp::AccountsToUpdateMintLp, price_exact_in::AccountsToUpdatePriceExactIn,
    redeem_lp::AccountsToUpdateRedeemLp, UpdatePricingProg,
};
use inf1_std::inf1_ctl_core::{
    accounts::pool_state::PoolStatePacked,
    keys::{LST_STATE_LIST_ID, POOL_STATE_ID},
};
use inf1_svc_ag_std::update::{AccountsToUpdateSvc, UpdateSvc};
use wasm_bindgen::prelude::*;

use crate::{
    acc_deser_err,
    err::{unknown_pp_err, InfError},
    interface::{Account, AccountMap, B58PK},
    missing_acc_err,
    pda::controller::create_raw_pool_reserves_ata,
    pricing::Pricing,
    trade::{Pair, PkPair},
    utils::{balance_from_token_acc_data, token_supply_from_mint_data, try_find_lst_state},
    Inf, Reserves,
};

/// Returned accounts are deduped
///
/// @throws
#[wasm_bindgen(js_name = accountsToUpdateForTrade)]
pub fn accounts_to_update_for_trade(
    inf: &mut Inf,
    PkPair(Pair {
        inp: Bs58Array(inp),
        out: Bs58Array(out),
    }): &PkPair,
) -> Result<Box<[B58PK]>, InfError> {
    let mut res = vec![B58PK::new(POOL_STATE_ID), B58PK::new(LST_STATE_LIST_ID)];

    let lp_token_mint = inf.pool.lp_token_mint;
    if *out == lp_token_mint {
        // add liquidity
        let pricing_accs = inf.pricing.0.accounts_to_update_mint_lp();
        let (_i, lst_state) = try_find_lst_state(inf.lst_state_list(), inp)?;
        let (calc, _) = inf.try_get_or_init_lst(&lst_state)?;
        res.extend(
            calc.accounts_to_update_svc()
                .chain(pricing_accs)
                .chain([
                    lp_token_mint,
                    create_raw_pool_reserves_ata(inp, lst_state.pool_reserves_bump),
                ])
                .map(B58PK::new),
        );
    } else if *inp == lp_token_mint {
        // remove liquidity
        let pricing_accs = inf.pricing.0.accounts_to_update_redeem_lp();
        let (_i, lst_state) = try_find_lst_state(inf.lst_state_list(), out)?;
        let (calc, _) = inf.try_get_or_init_lst(&lst_state)?;
        res.extend(
            calc.accounts_to_update_svc()
                .chain(pricing_accs)
                .chain([
                    lp_token_mint,
                    create_raw_pool_reserves_ata(out, lst_state.pool_reserves_bump),
                ])
                .map(B58PK::new),
        );
    } else {
        // swap
        for mint in [inp, out] {
            let (_i, lst_state) = try_find_lst_state(inf.lst_state_list(), mint)?;
            let (calc, _) = inf.try_get_or_init_lst(&lst_state)?;
            let reserves_addr = create_raw_pool_reserves_ata(mint, lst_state.pool_reserves_bump);
            res.extend(
                calc.accounts_to_update_svc()
                    .chain(iter::once(reserves_addr))
                    .map(B58PK::new),
            );
        }
        res.extend(
            inf.pricing
                .0
                .accounts_to_update_price_exact_in(&Pair { inp, out })
                .map(B58PK::new),
        );
    };

    res.sort();
    res.dedup();
    Ok(res.into_boxed_slice())
}

// TODO: this currently assumes accounts required for ExactIn
// and ExactOut are the same, tho this might not be the case for future pricing programs
/// @throws
#[wasm_bindgen(js_name = updateForTrade)]
pub fn update_for_trade(
    inf: &mut Inf,
    PkPair(Pair {
        inp: Bs58Array(inp),
        out: Bs58Array(out),
    }): &PkPair,
    account_map: &AccountMap,
) -> Result<(), InfError> {
    let AccountMap(fetched) = account_map;
    inf.update_ctl_accounts(fetched)?;
    if *out == inf.pool.lp_token_mint {
        // add liquidity
        inf.update_lp_token_supply(fetched)?;
        inf.update_lst(inp, account_map)?;
        inf.pricing.0.update_mint_lp(account_map)?;
    } else if *inp == inf.pool.lp_token_mint {
        // remove liquidity
        inf.update_lp_token_supply(fetched)?;
        inf.update_lst(out, account_map)?;
        inf.pricing.0.update_redeem_lp(account_map)?;
    } else {
        // swap
        [inp, out]
            .iter()
            .try_for_each::<_, Result<(), InfError>>(|mint| inf.update_lst(mint, account_map))?;
        inf.pricing
            .0
            .update_price_exact_in(&Pair { inp, out }, account_map)?;
    };

    Ok(())
}

impl Inf {
    fn update_ctl_accounts(&mut self, fetched: &HashMap<B58PK, Account>) -> Result<(), InfError> {
        let [pool_state_acc, lst_state_list_acc] = [POOL_STATE_ID, LST_STATE_LIST_ID].map(|pk| {
            fetched
                .get(&B58PK::new(pk))
                .ok_or_else(|| missing_acc_err(&pk))
        });
        let pool_state_acc = pool_state_acc?;
        let lst_state_list_acc = lst_state_list_acc?;

        let pool = PoolStatePacked::of_acc_data(&pool_state_acc.data)
            .ok_or_else(|| acc_deser_err(&POOL_STATE_ID))?
            .into_pool_state();

        if *self.pricing.0 .0.ty().program_id() != pool.pricing_program {
            self.pricing = Pricing::try_default_from_program_id(&pool.pricing_program)
                .ok_or_else(|| unknown_pp_err(&pool.pricing_program))?;
        }

        self.pool = pool;
        self.lst_state_list_data = lst_state_list_acc.data.as_slice().into();

        // TODO: maybe cleanup removed LSTs from self.lsts?

        Ok(())
    }

    fn update_lp_token_supply(
        &mut self,
        fetched: &HashMap<B58PK, Account>,
    ) -> Result<(), InfError> {
        let lp_mint_acc = fetched
            .get(&B58PK::new(self.pool.lp_token_mint))
            .ok_or_else(|| missing_acc_err(&self.pool.lp_token_mint))?;
        let lp_token_supply = token_supply_from_mint_data(&lp_mint_acc.data)
            .ok_or_else(|| acc_deser_err(&self.pool.lp_token_mint))?;

        self.lp_token_supply = Some(lp_token_supply);

        Ok(())
    }

    fn update_lst(&mut self, mint: &[u8; 32], fetched: &AccountMap) -> Result<(), InfError> {
        let (_i, lst_state) = try_find_lst_state(self.lst_state_list(), mint)?;
        let reserves_addr = create_raw_pool_reserves_ata(mint, lst_state.pool_reserves_bump);
        let (calc, reserves) = self.try_get_or_init_lst(&lst_state)?;
        calc.update_svc(fetched)?;
        Reserves::update(reserves, reserves_addr, &fetched.0)
    }
}

impl Reserves {
    fn update(
        reserves: &mut Option<Reserves>,
        reserves_addr: [u8; 32],
        fetched: &HashMap<B58PK, Account>,
    ) -> Result<(), InfError> {
        let token_acc = fetched
            .get(&B58PK::new(reserves_addr))
            .ok_or_else(|| missing_acc_err(&reserves_addr))?;
        *reserves = Some(Reserves {
            balance: balance_from_token_acc_data(&token_acc.data)
                .ok_or_else(|| acc_deser_err(&reserves_addr))?,
        });
        Ok(())
    }
}
