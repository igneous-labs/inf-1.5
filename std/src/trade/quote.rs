#![allow(deprecated)]

use inf1_core::{
    inf1_ctl_core::typedefs::{lst_state::LstState, u8bool::U8Bool},
    inf1_pp_core::{
        pair::Pair,
        traits::{
            collection::{PriceExactInCol, PriceExactOutCol},
            deprecated::{PriceLpTokensToMintCol, PriceLpTokensToRedeemCol},
        },
    },
    quote::{
        liquidity::{
            add::{quote_add_liq, AddLiqQuote, AddLiqQuoteArgs, AddLiqQuoteErr},
            remove::{quote_remove_liq, RemoveLiqQuote, RemoveLiqQuoteArgs, RemoveLiqQuoteErr},
        },
        swap::{
            err::SwapQuoteErr, exact_in::quote_exact_in, exact_out::quote_exact_out, SwapQuote,
            SwapQuoteArgs,
        },
    },
};
use inf1_svc_ag_std::calc::{SvcCalcAg, SvcCalcAgErr};

use crate::{
    err::InfErr,
    trade::{Trade, TradeLimitTy},
    utils::manual_sync_sol_value,
    Inf,
};

pub type TradeQuote = Trade<AddLiqQuote, RemoveLiqQuote, SwapQuote, SwapQuote>;

impl<F, C: Fn(&[&[u8]], &[u8; 32]) -> Option<[u8; 32]>> Inf<F, C> {
    #[inline]
    pub fn quote_trade_mut(
        &mut self,
        pair: &Pair<&[u8; 32]>,
        amt: u64,
        limit_ty: TradeLimitTy,
    ) -> Result<TradeQuote, InfErr> {
        match limit_ty {
            TradeLimitTy::ExactOut => {
                // currently only swap is supported for ExactOut
                self.quote_exact_out_mut(pair, amt).map(Trade::SwapExactOut)
            }
            TradeLimitTy::ExactIn => {
                let lp_token_mint = self.pool.lp_token_mint;
                if *pair.out == lp_token_mint {
                    self.quote_add_liq_mut(pair.inp, amt)
                        .map(Trade::AddLiquidity)
                } else if *pair.inp == lp_token_mint {
                    self.quote_remove_liq_mut(pair.out, amt)
                        .map(Trade::RemoveLiquidity)
                } else {
                    self.quote_exact_in_mut(pair, amt).map(Trade::SwapExactIn)
                }
            }
        }
    }

    #[inline]
    pub fn quote_trade(
        &self,
        pair: &Pair<&[u8; 32]>,
        amt: u64,
        limit_ty: TradeLimitTy,
    ) -> Result<TradeQuote, InfErr> {
        match limit_ty {
            TradeLimitTy::ExactOut => {
                // currently only swap is supported for ExactOut
                self.quote_exact_out(pair, amt).map(Trade::SwapExactOut)
            }
            TradeLimitTy::ExactIn => {
                let lp_token_mint = self.pool.lp_token_mint;
                if *pair.out == lp_token_mint {
                    self.quote_add_liq(pair.inp, amt).map(Trade::AddLiquidity)
                } else if *pair.inp == lp_token_mint {
                    self.quote_remove_liq(pair.out, amt)
                        .map(Trade::RemoveLiquidity)
                } else {
                    self.quote_exact_in(pair, amt).map(Trade::SwapExactIn)
                }
            }
        }
    }

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

        let reserves = self.lst_reserves.get(mint).ok_or_else(|| {
            self.create_pool_reserves_ata(mint, lst_state.pool_reserves_bump)
                .map_or_else(|| InfErr::NoValidPda, |pk| InfErr::MissingAcc { pk })
        })?;

        // need to perform a manual SyncSolValue of inp mint first
        // in case pool_total_sol_value is stale
        let pool_total_sol_value =
            manual_sync_sol_value(self.pool.total_sol_value, lst_state, calc, reserves.balance)
                .map_err(map_inp_calc_err)?
                .exec();

        Ok((lp_token_supply, pool_total_sol_value, reserves.balance))
    }

    #[inline]
    fn quote_add_liq_inner(
        &self,
        inp_mint: &[u8; 32],
        amt: u64,
        inp_lst_state: &LstState,
        inp_calc: &SvcCalcAg,
    ) -> Result<AddLiqQuote, InfErr> {
        if U8Bool(&inp_lst_state.is_input_disabled).to_bool() {
            return Err(InfErr::SwapQuote(SwapQuoteErr::InpDisabled));
        }
        let (lp_token_supply, pool_total_sol_value, _reserves) =
            self.quote_liq_common(inp_mint, inp_lst_state, inp_calc, |e| {
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
    pub fn quote_add_liq(&self, inp_mint: &[u8; 32], amt: u64) -> Result<AddLiqQuote, InfErr> {
        let (inp_lst_state, inp_calc) = self.lst_state_and_calc(inp_mint)?;
        self.quote_add_liq_inner(inp_mint, amt, &inp_lst_state, &inp_calc)
    }

    #[inline]
    pub fn quote_add_liq_mut(
        &mut self,
        inp_mint: &[u8; 32],
        amt: u64,
    ) -> Result<AddLiqQuote, InfErr> {
        let (inp_lst_state, inp_calc) = self.lst_state_and_calc_mut(inp_mint)?;
        self.quote_add_liq_inner(inp_mint, amt, &inp_lst_state, &inp_calc)
    }

    #[inline]
    fn quote_remove_liq_inner(
        &self,
        out_mint: &[u8; 32],
        amt: u64,
        out_lst_state: &LstState,
        out_calc: &SvcCalcAg,
    ) -> Result<RemoveLiqQuote, InfErr> {
        let (lp_token_supply, pool_total_sol_value, out_reserves) =
            self.quote_liq_common(out_mint, out_lst_state, out_calc, |e| {
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
    pub fn quote_remove_liq(
        &self,
        out_mint: &[u8; 32],
        amt: u64,
    ) -> Result<RemoveLiqQuote, InfErr> {
        let (out_lst_state, out_calc) = self.lst_state_and_calc(out_mint)?;
        self.quote_remove_liq_inner(out_mint, amt, &out_lst_state, &out_calc)
    }

    #[inline]
    pub fn quote_remove_liq_mut(
        &mut self,
        out_mint: &[u8; 32],
        amt: u64,
    ) -> Result<RemoveLiqQuote, InfErr> {
        let (out_lst_state, out_calc) = self.lst_state_and_calc_mut(out_mint)?;
        self.quote_remove_liq_inner(out_mint, amt, &out_lst_state, &out_calc)
    }

    #[inline]
    fn quote_exact_in_inner(
        &self,
        pair: &Pair<&[u8; 32]>,
        amt: u64,
        Pair {
            inp: (inp_lst_state, inp_calc),
            out: (out_lst_state, out_calc),
        }: &Pair<(LstState, SvcCalcAg)>,
    ) -> Result<SwapQuote, InfErr> {
        if U8Bool(&inp_lst_state.is_input_disabled).to_bool() {
            return Err(InfErr::SwapQuote(SwapQuoteErr::InpDisabled));
        }
        let out_reserves = self.reserves_balance_checked(pair.out, out_lst_state)?;
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
    pub fn quote_exact_in(&self, pair: &Pair<&[u8; 32]>, amt: u64) -> Result<SwapQuote, InfErr> {
        let lsc = pair.try_map(|mint| self.lst_state_and_calc(mint))?;
        self.quote_exact_in_inner(pair, amt, &lsc)
    }

    #[inline]
    pub fn quote_exact_in_mut(
        &mut self,
        pair: &Pair<&[u8; 32]>,
        amt: u64,
    ) -> Result<SwapQuote, InfErr> {
        let lsc = pair.try_map(|mint| self.lst_state_and_calc_mut(mint))?;
        self.quote_exact_in_inner(pair, amt, &lsc)
    }

    #[inline]
    fn quote_exact_out_inner(
        &self,
        pair: &Pair<&[u8; 32]>,
        amt: u64,
        Pair {
            inp: (inp_lst_state, inp_calc),
            out: (out_lst_state, out_calc),
        }: &Pair<(LstState, SvcCalcAg)>,
    ) -> Result<SwapQuote, InfErr> {
        if U8Bool(&inp_lst_state.is_input_disabled).to_bool() {
            return Err(InfErr::SwapQuote(SwapQuoteErr::InpDisabled));
        }
        let out_reserves = self.reserves_balance_checked(pair.out, out_lst_state)?;
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
    pub fn quote_exact_out(&self, pair: &Pair<&[u8; 32]>, amt: u64) -> Result<SwapQuote, InfErr> {
        let lsc = pair.try_map(|mint| self.lst_state_and_calc(mint))?;
        self.quote_exact_out_inner(pair, amt, &lsc)
    }

    #[inline]
    pub fn quote_exact_out_mut(
        &mut self,
        pair: &Pair<&[u8; 32]>,
        amt: u64,
    ) -> Result<SwapQuote, InfErr> {
        let lsc = pair.try_map(|mint| self.lst_state_and_calc_mut(mint))?;
        self.quote_exact_out_inner(pair, amt, &lsc)
    }
}
