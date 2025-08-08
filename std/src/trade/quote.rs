#![allow(deprecated)]

use inf1_core::{
    inf1_ctl_core::typedefs::lst_state::LstState,
    inf1_pp_core::{
        pair::Pair,
        traits::{
            collection::{PriceExactInCol, PriceExactOutCol},
            deprecated::{PriceLpTokensToMintCol, PriceLpTokensToRedeemCol},
        },
    },
    inf1_svc_core::traits::SolValCalc,
    quote::{
        liquidity::{
            add::{quote_add_liq, AddLiqQuote, AddLiqQuoteArgs, AddLiqQuoteErr},
            remove::{quote_remove_liq, RemoveLiqQuote, RemoveLiqQuoteArgs, RemoveLiqQuoteErr},
        },
        swap::{exact_in::quote_exact_in, exact_out::quote_exact_out, SwapQuote, SwapQuoteArgs},
    },
    sync::SyncSolVal,
};
use inf1_svc_ag_std::calc::{SvcCalcAg, SvcCalcAgErr};

use crate::{
    err::InfErr,
    utils::{try_find_lst_state, try_map_pair},
    Inf,
};

impl<F, C> Inf<F, C> {
    fn lst_state_and_calc(&self, mint: &[u8; 32]) -> Result<(LstState, SvcCalcAg), InfErr> {
        let (_i, lst_state) = try_find_lst_state(self.lst_state_list(), mint)?;
        let calc = self
            .try_get_lst_svc(mint)?
            .as_sol_val_calc()
            .ok_or(InfErr::MissingSvcData { mint: *mint })?
            .to_owned_copy();
        Ok((lst_state, calc))
    }

    fn lst_state_and_calc_mut(&mut self, mint: &[u8; 32]) -> Result<(LstState, SvcCalcAg), InfErr> {
        let (_i, lst_state) = try_find_lst_state(self.lst_state_list(), mint)?;
        let calc = self
            .try_get_or_init_lst_svc_mut(&lst_state)?
            .as_sol_val_calc()
            .ok_or(InfErr::MissingSvcData { mint: *mint })?
            .to_owned_copy();
        Ok((lst_state, calc))
    }
}

impl<F, C: Fn(&[&[u8]], &[u8; 32]) -> Option<[u8; 32]>> Inf<F, C> {
    /// Returns `(lp_token_supply, pool_total_sol_value, out_reserves_balance, calc)`
    fn quote_liq_common(
        &self,
        mint: &[u8; 32],
        lst_state: &LstState,
        calc: &SvcCalcAg,
        map_inp_calc_err: impl Fn(SvcCalcAgErr) -> InfErr,
    ) -> Result<(u64, u64, u64), InfErr> {
        let lp_token_supply = self.lp_token_supply.ok_or(InfErr::MissingAcc {
            pk: self.pool.lp_token_mint,
        })?;

        let reserves = self.try_get_lst_reserves(mint).ok_or_else(|| {
            self.create_pool_reserves_ata(mint, lst_state.pool_reserves_bump)
                .map_or_else(|| InfErr::NoValidPda, |pk| InfErr::MissingAcc { pk })
        })?;

        // need to perform a manual SyncSolValue of inp mint first
        // in case pool_total_sol_value is stale
        let old_sol_val = lst_state.sol_value;
        let new_sol_val_range = calc
            .lst_to_sol(reserves.balance)
            .map_err(map_inp_calc_err)?;
        let new_sol_val = new_sol_val_range.start();
        let pool_total_sol_value = SyncSolVal {
            pool_total: self.pool.total_sol_value,
            lst_old: old_sol_val,
            lst_new: *new_sol_val,
        }
        .exec();

        Ok((lp_token_supply, pool_total_sol_value, reserves.balance))
    }

    #[inline]
    pub fn quote_add_liq(&self, inp_mint: &[u8; 32], amt: u64) -> Result<AddLiqQuote, InfErr> {
        let (inp_lst_state, inp_calc) = self.lst_state_and_calc(inp_mint)?;
        let (lp_token_supply, pool_total_sol_value, _reserves) =
            self.quote_liq_common(inp_mint, &inp_lst_state, &inp_calc, |e| {
                InfErr::AddLiqQuote(AddLiqQuoteErr::InpCalc(e))
            })?;
        let pricing = self
            .pricing
            .price_lp_tokens_to_mint_for(inp_mint)
            .map_err(InfErr::PricingProg)?;
        quote_add_liq(AddLiqQuoteArgs {
            amt,
            lp_token_supply,
            pool_total_sol_value,
            lp_protocol_fee_bps: self.pool.lp_protocol_fee_bps,
            inp_mint: *inp_mint,
            lp_mint: self.pool.lp_token_mint,
            inp_calc,
            pricing,
        })
        .map_err(InfErr::AddLiqQuote)
    }

    #[inline]
    pub fn quote_add_liq_mut(
        &mut self,
        inp_mint: &[u8; 32],
        amt: u64,
    ) -> Result<AddLiqQuote, InfErr> {
        let (inp_lst_state, inp_calc) = self.lst_state_and_calc_mut(inp_mint)?;
        let (lp_token_supply, pool_total_sol_value, _reserves) =
            self.quote_liq_common(inp_mint, &inp_lst_state, &inp_calc, |e| {
                InfErr::AddLiqQuote(AddLiqQuoteErr::InpCalc(e))
            })?;
        let pricing = self
            .pricing
            .price_lp_tokens_to_mint_for(inp_mint)
            .map_err(InfErr::PricingProg)?;
        quote_add_liq(AddLiqQuoteArgs {
            amt,
            lp_token_supply,
            pool_total_sol_value,
            lp_protocol_fee_bps: self.pool.lp_protocol_fee_bps,
            inp_mint: *inp_mint,
            lp_mint: self.pool.lp_token_mint,
            inp_calc,
            pricing,
        })
        .map_err(InfErr::AddLiqQuote)
    }

    #[inline]
    pub fn quote_remove_liq(
        &self,
        out_mint: &[u8; 32],
        amt: u64,
    ) -> Result<RemoveLiqQuote, InfErr> {
        let (out_lst_state, out_calc) = self.lst_state_and_calc(out_mint)?;
        let (lp_token_supply, pool_total_sol_value, out_reserves) =
            self.quote_liq_common(out_mint, &out_lst_state, &out_calc, |e| {
                InfErr::RemoveLiqQuote(RemoveLiqQuoteErr::OutCalc(e))
            })?;
        let pricing = self
            .pricing
            .price_lp_tokens_to_redeem_for(out_mint)
            .map_err(InfErr::PricingProg)?;
        quote_remove_liq(RemoveLiqQuoteArgs {
            amt,
            lp_token_supply,
            pool_total_sol_value,
            lp_protocol_fee_bps: self.pool.lp_protocol_fee_bps,
            out_mint: *out_mint,
            lp_mint: self.pool.lp_token_mint,
            out_calc,
            pricing,
            out_reserves,
        })
        .map_err(InfErr::RemoveLiqQuote)
    }

    #[inline]
    pub fn quote_remove_liq_mut(
        &mut self,
        out_mint: &[u8; 32],
        amt: u64,
    ) -> Result<RemoveLiqQuote, InfErr> {
        let (out_lst_state, out_calc) = self.lst_state_and_calc_mut(out_mint)?;
        let (lp_token_supply, pool_total_sol_value, out_reserves) =
            self.quote_liq_common(out_mint, &out_lst_state, &out_calc, |e| {
                InfErr::RemoveLiqQuote(RemoveLiqQuoteErr::OutCalc(e))
            })?;
        let pricing = self
            .pricing
            .price_lp_tokens_to_redeem_for(out_mint)
            .map_err(InfErr::PricingProg)?;
        quote_remove_liq(RemoveLiqQuoteArgs {
            amt,
            lp_token_supply,
            pool_total_sol_value,
            lp_protocol_fee_bps: self.pool.lp_protocol_fee_bps,
            out_mint: *out_mint,
            lp_mint: self.pool.lp_token_mint,
            out_calc,
            pricing,
            out_reserves,
        })
        .map_err(InfErr::RemoveLiqQuote)
    }

    fn reserves_balance_checked(
        &self,
        mint: &[u8; 32],
        lst_state: &LstState,
    ) -> Result<u64, InfErr> {
        let out_reserves_balance = self
            .try_get_lst_reserves(mint)
            .ok_or_else(|| {
                self.create_pool_reserves_ata(mint, lst_state.pool_reserves_bump)
                    .map_or_else(|| InfErr::NoValidPda, |pk| InfErr::MissingAcc { pk })
            })?
            .balance;

        // TODO: we dont manual sync sol value for swap (unlike add/remove liq) right now
        // because no pricing progs / sol val calcs rely on the pool's total sol value.
        // This may change in the future

        Ok(out_reserves_balance)
    }

    #[inline]
    pub fn quote_exact_in(&self, pair: &Pair<&[u8; 32]>, amt: u64) -> Result<SwapQuote, InfErr> {
        let Pair {
            inp: (_, inp_calc),
            out: (out_lst_state, out_calc),
        } = try_map_pair(*pair, |mint| self.lst_state_and_calc(mint))?;
        let out_reserves = self.reserves_balance_checked(pair.out, &out_lst_state)?;
        let pricing = self
            .pricing
            .price_exact_in_for(pair)
            .map_err(InfErr::PricingProg)?;
        quote_exact_in(SwapQuoteArgs {
            amt,
            inp_mint: *pair.inp,
            out_mint: *pair.out,
            inp_calc,
            out_calc,
            pricing,
            out_reserves,
            trading_protocol_fee_bps: self.pool.trading_protocol_fee_bps,
        })
        .map_err(InfErr::SwapQuote)
    }

    #[inline]
    pub fn quote_exact_in_mut(
        &mut self,
        pair: &Pair<&[u8; 32]>,
        amt: u64,
    ) -> Result<SwapQuote, InfErr> {
        let Pair {
            inp: (_, inp_calc),
            out: (out_lst_state, out_calc),
        } = try_map_pair(*pair, |mint| self.lst_state_and_calc_mut(mint))?;
        let out_reserves = self.reserves_balance_checked(pair.out, &out_lst_state)?;
        let pricing = self
            .pricing
            .price_exact_in_for(pair)
            .map_err(InfErr::PricingProg)?;
        quote_exact_in(SwapQuoteArgs {
            amt,
            inp_mint: *pair.inp,
            out_mint: *pair.out,
            inp_calc,
            out_calc,
            pricing,
            out_reserves,
            trading_protocol_fee_bps: self.pool.trading_protocol_fee_bps,
        })
        .map_err(InfErr::SwapQuote)
    }

    #[inline]
    pub fn quote_exact_out(&self, pair: &Pair<&[u8; 32]>, amt: u64) -> Result<SwapQuote, InfErr> {
        let Pair {
            inp: (_, inp_calc),
            out: (out_lst_state, out_calc),
        } = try_map_pair(*pair, |mint| self.lst_state_and_calc(mint))?;
        let out_reserves = self.reserves_balance_checked(pair.out, &out_lst_state)?;
        let pricing = self
            .pricing
            .price_exact_out_for(pair)
            .map_err(InfErr::PricingProg)?;
        quote_exact_out(SwapQuoteArgs {
            amt,
            inp_mint: *pair.inp,
            out_mint: *pair.out,
            inp_calc,
            out_calc,
            pricing,
            out_reserves,
            trading_protocol_fee_bps: self.pool.trading_protocol_fee_bps,
        })
        .map_err(InfErr::SwapQuote)
    }

    #[inline]
    pub fn quote_exact_out_mut(
        &mut self,
        pair: &Pair<&[u8; 32]>,
        amt: u64,
    ) -> Result<SwapQuote, InfErr> {
        let Pair {
            inp: (_, inp_calc),
            out: (out_lst_state, out_calc),
        } = try_map_pair(*pair, |mint| self.lst_state_and_calc_mut(mint))?;
        let out_reserves = self.reserves_balance_checked(pair.out, &out_lst_state)?;
        let pricing = self
            .pricing
            .price_exact_out_for(pair)
            .map_err(InfErr::PricingProg)?;
        quote_exact_out(SwapQuoteArgs {
            amt,
            inp_mint: *pair.inp,
            out_mint: *pair.out,
            inp_calc,
            out_calc,
            pricing,
            out_reserves,
            trading_protocol_fee_bps: self.pool.trading_protocol_fee_bps,
        })
        .map_err(InfErr::SwapQuote)
    }
}
