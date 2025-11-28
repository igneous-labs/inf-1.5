use inf1_core::{
    inf1_pp_core::{
        pair::Pair,
        traits::collection::{PriceExactInCol, PriceExactOutCol},
    },
    quote::swap::{exact_in::quote_exact_in, exact_out::quote_exact_out, SwapQuote, SwapQuoteArgs},
};

use crate::{
    err::InfErr,
    trade::{Trade, TradeLimitTy},
    Inf,
};

pub type TradeQuote = Trade<SwapQuote, SwapQuote>;

impl<F, C: Fn(&[&[u8]], &[u8; 32]) -> Option<[u8; 32]>> Inf<F, C> {
    #[inline]
    pub fn quote_trade_mut(
        &mut self,
        pair: &Pair<&[u8; 32]>,
        amt: u64,
        limit_ty: TradeLimitTy,
    ) -> Result<TradeQuote, InfErr> {
        match limit_ty {
            TradeLimitTy::ExactOut(_) => self.quote_exact_out_mut(pair, amt).map(Trade::ExactOut),
            TradeLimitTy::ExactIn(_) => self.quote_exact_in_mut(pair, amt).map(Trade::ExactIn),
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
            TradeLimitTy::ExactOut(_) => self.quote_exact_out(pair, amt).map(Trade::ExactOut),
            TradeLimitTy::ExactIn(_) => self.quote_exact_in(pair, amt).map(Trade::ExactIn),
        }
    }

    #[inline]
    pub fn quote_exact_in(&self, pair: &Pair<&[u8; 32]>, amt: u64) -> Result<SwapQuote, InfErr> {
        let Pair {
            inp: (_, inp_calc),
            out: (out_lst_state, out_calc),
        } = pair.try_map(|mint| self.lst_state_and_calc(mint))?;
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
        } = pair.try_map(|mint| self.lst_state_and_calc_mut(mint))?;
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
        } = pair.try_map(|mint| self.lst_state_and_calc(mint))?;
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
        } = pair.try_map(|mint| self.lst_state_and_calc_mut(mint))?;
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
