use inf1_core::{
    inf1_pp_core::{
        pair::Pair,
        traits::collection::{PriceExactInCol, PriceExactOutCol},
    },
    quote::{
        swap::{exact_in::quote_exact_in, exact_out::quote_exact_out, QuoteArgs},
        Quote,
    },
};
use inf1_svc_ag_std::SvcAg;

use crate::{err::InfErr, trade::TradeLimitTy, Inf};

impl<F, C: Fn(&[&[u8]], &[u8; 32]) -> Option<[u8; 32]>> Inf<F, C> {
    #[inline]
    pub fn quote_trade_mut(
        &mut self,
        pair: &Pair<&[u8; 32]>,
        amt: u64,
        slot_lookahead: u64,
        limit_ty: TradeLimitTy,
    ) -> Result<Quote, InfErr> {
        match limit_ty {
            TradeLimitTy::ExactOut(_) => self.quote_exact_out_mut(pair, amt, slot_lookahead),
            TradeLimitTy::ExactIn(_) => self.quote_exact_in_mut(pair, amt, slot_lookahead),
        }
    }

    #[inline]
    pub fn quote_trade(
        &self,
        pair: &Pair<&[u8; 32]>,
        amt: u64,
        slot_lookahead: u64,
        limit_ty: TradeLimitTy,
    ) -> Result<Quote, InfErr> {
        match limit_ty {
            TradeLimitTy::ExactOut(_) => self.quote_exact_out(pair, amt, slot_lookahead),
            TradeLimitTy::ExactIn(_) => self.quote_exact_in(pair, amt, slot_lookahead),
        }
    }

    #[inline]
    pub fn quote_exact_in(
        &self,
        pair: &Pair<&[u8; 32]>,
        amt: u64,
        slot_lookahead: u64,
    ) -> Result<Quote, InfErr> {
        let Pair {
            inp: (_, inp_calc),
            out: (out_reserves, out_calc),
        } = pair.try_map(|mint| {
            Ok(if mint == self.pool.lp_token_mint() {
                let calc = self.inf_calc(slot_lookahead)?;
                (u64::MAX, SvcAg::Inf(calc))
            } else {
                let (lst_state, calc) = self.lst_state_and_calc(mint)?;
                let reserves = self.reserves_balance_checked(&lst_state)?;
                (reserves, calc)
            })
        })?;
        let pricing = self
            .pricing
            .price_exact_in_for(pair)
            .map_err(InfErr::PricingProg)?;
        quote_exact_in(&QuoteArgs {
            amt,
            inp_mint: *pair.inp,
            out_mint: *pair.out,
            inp_calc,
            out_calc,
            pricing,
            out_reserves,
        })
        .map_err(InfErr::SwapQuote)
    }

    #[inline]
    pub fn quote_exact_in_mut(
        &mut self,
        pair: &Pair<&[u8; 32]>,
        amt: u64,
        slot_lookahead: u64,
    ) -> Result<Quote, InfErr> {
        let Pair {
            inp: (_, inp_calc),
            out: (out_reserves, out_calc),
        } = pair.try_map(|mint| {
            Ok(if mint == self.pool.lp_token_mint() {
                let calc = self.inf_calc(slot_lookahead)?;
                (u64::MAX, SvcAg::Inf(calc))
            } else {
                let (lst_state, calc) = self.lst_state_and_calc_mut(mint)?;
                let reserves = self.reserves_balance_checked(&lst_state)?;
                (reserves, calc)
            })
        })?;
        let pricing = self
            .pricing
            .price_exact_in_for(pair)
            .map_err(InfErr::PricingProg)?;
        quote_exact_in(&QuoteArgs {
            amt,
            inp_mint: *pair.inp,
            out_mint: *pair.out,
            inp_calc,
            out_calc,
            pricing,
            out_reserves,
        })
        .map_err(InfErr::SwapQuote)
    }

    #[inline]
    pub fn quote_exact_out(
        &self,
        pair: &Pair<&[u8; 32]>,
        amt: u64,
        slot_lookahead: u64,
    ) -> Result<Quote, InfErr> {
        let Pair {
            inp: (_, inp_calc),
            out: (out_reserves, out_calc),
        } = pair.try_map(|mint| {
            Ok(if mint == self.pool.lp_token_mint() {
                let calc = self.inf_calc(slot_lookahead)?;
                (u64::MAX, SvcAg::Inf(calc))
            } else {
                let (lst_state, calc) = self.lst_state_and_calc(mint)?;
                let reserves = self.reserves_balance_checked(&lst_state)?;
                (reserves, calc)
            })
        })?;
        let pricing = self
            .pricing
            .price_exact_out_for(pair)
            .map_err(InfErr::PricingProg)?;
        quote_exact_out(&QuoteArgs {
            amt,
            inp_mint: *pair.inp,
            out_mint: *pair.out,
            inp_calc,
            out_calc,
            pricing,
            out_reserves,
        })
        .map_err(InfErr::SwapQuote)
    }

    #[inline]
    pub fn quote_exact_out_mut(
        &mut self,
        pair: &Pair<&[u8; 32]>,
        amt: u64,
        slot_lookahead: u64,
    ) -> Result<Quote, InfErr> {
        let Pair {
            inp: (_, inp_calc),
            out: (out_reserves, out_calc),
        } = pair.try_map(|mint| {
            Ok(if mint == self.pool.lp_token_mint() {
                let calc = self.inf_calc(slot_lookahead)?;
                (u64::MAX, SvcAg::Inf(calc))
            } else {
                let (lst_state, calc) = self.lst_state_and_calc_mut(mint)?;
                let reserves = self.reserves_balance_checked(&lst_state)?;
                (reserves, calc)
            })
        })?;
        let pricing = self
            .pricing
            .price_exact_out_for(pair)
            .map_err(InfErr::PricingProg)?;
        quote_exact_out(&QuoteArgs {
            amt,
            inp_mint: *pair.inp,
            out_mint: *pair.out,
            inp_calc,
            out_calc,
            pricing,
            out_reserves,
        })
        .map_err(InfErr::SwapQuote)
    }
}
