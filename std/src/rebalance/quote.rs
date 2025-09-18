use inf1_core::quote::rebalance::{quote_rebalance_exact_out, RebalanceQuote, RebalanceQuoteArgs};
use inf1_pp_ag_std::update::all::Pair;

use crate::{err::InfErr, Inf};

impl<F, C: Fn(&[&[u8]], &[u8; 32]) -> Option<[u8; 32]>> Inf<F, C> {
    #[inline]
    pub fn quote_rebalance_exact_out_mut(
        &mut self,
        pair: &Pair<&[u8; 32]>,
        amt: u64,
    ) -> Result<RebalanceQuote, InfErr> {
        let Pair {
            inp: (inp_calc, inp_reserves),
            out: (out_calc, out_reserves),
        } = pair.try_map(|mint| {
            let (state, calc) = self.lst_state_and_calc_mut(mint)?;
            let reserves = self.reserves_balance_checked(mint, &state)?;
            Ok((calc, reserves))
        })?;
        quote_rebalance_exact_out(RebalanceQuoteArgs {
            amt,
            inp_reserves,
            out_reserves,
            inp_mint: *pair.inp,
            out_mint: *pair.out,
            inp_calc,
            out_calc,
        })
        .map_err(InfErr::RebalanceQuote)
    }
}
