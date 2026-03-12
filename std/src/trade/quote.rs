use inf1_core::{
    inf1_ctl_core::typedefs::u8bool::U8Bool,
    inf1_pp_core::{
        pair::{Pair, PairMbr},
        traits::collection::{PriceExactInCol, PriceExactOutCol},
    },
    quote::{
        swap::{err::QuoteErr, exact_in::quote_exact_in, exact_out::quote_exact_out, QuoteArgs},
        Quote,
    },
};
use inf1_svc_ag_std::{calc::SvcCalcAg, SvcAg};

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
    fn reserves_and_calc(
        &self,
        mint: PairMbr<&[u8; 32]>,
        slot_lookahead: u64,
    ) -> Result<(u64, SvcCalcAg), InfErr> {
        let m = mint.as_ref_t();
        Ok(if *m == self.pool.lp_token_mint() {
            let calc = self.inf_calc(slot_lookahead)?;
            (u64::MAX, SvcAg::Inf(calc))
        } else {
            let (lst_state, calc) = self.lst_state_and_calc(m)?;
            if matches!(mint, PairMbr::Inp(_)) && U8Bool(&lst_state.is_input_disabled).to_bool() {
                return Err(InfErr::SwapQuote(QuoteErr::InpDisabled));
            }
            let reserves = self.reserves_balance_checked(&lst_state)?;
            (reserves, calc)
        })
    }

    #[inline]
    fn reserves_and_calc_mut(
        &mut self,
        mint: PairMbr<&[u8; 32]>,
        slot_lookahead: u64,
    ) -> Result<(u64, SvcCalcAg), InfErr> {
        let m = mint.as_ref_t();
        Ok(if *m == self.pool.lp_token_mint() {
            let calc = self.inf_calc(slot_lookahead)?;
            (u64::MAX, SvcAg::Inf(calc))
        } else {
            let (lst_state, calc) = self.lst_state_and_calc_mut(m)?;
            if matches!(mint, PairMbr::Inp(_)) && U8Bool(&lst_state.is_input_disabled).to_bool() {
                return Err(InfErr::SwapQuote(QuoteErr::InpDisabled));
            }
            let reserves = self.reserves_balance_checked(&lst_state)?;
            (reserves, calc)
        })
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
        } = pair.try_map_mbr(|mint| self.reserves_and_calc(mint, slot_lookahead))?;
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
        } = pair.try_map_mbr(|mint| self.reserves_and_calc_mut(mint, slot_lookahead))?;
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
        } = pair.try_map_mbr(|mint| self.reserves_and_calc(mint, slot_lookahead))?;
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
        } = pair.try_map_mbr(|mint| self.reserves_and_calc_mut(mint, slot_lookahead))?;
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
