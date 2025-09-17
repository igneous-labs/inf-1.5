use core::{error::Error, fmt::Display};

use inf1_svc_core::traits::SolValCalc;

use crate::err::NotEnoughLiquidityErr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RebalanceQuoteArgs<I, O> {
    pub amt: u64,

    /// Needed because we want to do
    /// `sol_value(post) - sol_value(pre)`
    /// instead of just
    /// `sol_value(amt)`
    /// in order to match onchain math exactly
    pub inp_reserves: u64,

    /// Needed because we want to do
    /// `sol_value(post) - sol_value(pre)`
    /// instead of just
    /// `sol_value(amt)`
    /// in order to match onchain math exactly
    pub out_reserves: u64,

    pub inp_mint: [u8; 32],

    pub out_mint: [u8; 32],

    pub inp_calc: I,

    pub out_calc: O,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RebalanceQuote {
    /// Amount of output tokens that will leave the pool in StartRebalance
    pub out: u64,

    /// Amount of input tokens that needs to enter the pool by EndRebalance
    pub inp: u64,

    pub out_mint: [u8; 32],
    pub inp_mint: [u8; 32],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RebalanceQuoteErr<I, O> {
    InpCalc(I),
    OutCalc(O),
    NotEnoughLiquidity(NotEnoughLiquidityErr),
    Overflow,
}

impl<I: Display, O: Display> Display for RebalanceQuoteErr<I, O> {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::OutCalc(e) => e.fmt(f),
            Self::Overflow => f.write_str("arithmetic overflow"),
            Self::NotEnoughLiquidity(e) => e.fmt(f),
            Self::InpCalc(e) => e.fmt(f),
        }
    }
}

// fully qualify core::fmt::Debug instead of importing so that .fmt() doesnt clash with Display
impl<I: core::fmt::Debug + Display, O: core::fmt::Debug + Display> Error
    for RebalanceQuoteErr<I, O>
{
}

pub type RebalanceQuoteResult<I, O> = Result<RebalanceQuote, RebalanceQuoteErr<I, O>>;

pub fn quote_rebalance_exact_out<I: SolValCalc, O: SolValCalc>(
    RebalanceQuoteArgs {
        amt,
        inp_mint,
        out_mint,
        inp_calc,
        out_calc,
        inp_reserves,
        out_reserves,
    }: RebalanceQuoteArgs<I, O>,
) -> RebalanceQuoteResult<I::Error, O::Error> {
    if amt > out_reserves {
        return Err(RebalanceQuoteErr::NotEnoughLiquidity(
            NotEnoughLiquidityErr {
                required: amt,
                available: out_reserves,
            },
        ));
    }
    let sol_value_out = {
        // unchecked-arith: limit checked above
        let [pre, post] = [out_reserves, out_reserves - amt].map(|lst| {
            out_calc
                .lst_to_sol(lst)
                .map_err(RebalanceQuoteErr::OutCalc)
                .map(|r| *r.start())
        });
        let pre = pre?;
        let post = post?;
        pre.checked_sub(post).ok_or(RebalanceQuoteErr::Overflow)?
    };
    let inp_pre_sol_value = *inp_calc
        .lst_to_sol(inp_reserves)
        .map_err(RebalanceQuoteErr::InpCalc)?
        .start();

    // want to find:
    // smallest `post - pre` s.t.
    // s(post) - s(pre) = x
    // where x = sol_value,
    // s(y) = lst_to_sol(y).start()
    //
    // s(post) = x + s(pre)

    let req_inp_post_sol_value = inp_pre_sol_value
        .checked_add(sol_value_out)
        .ok_or(RebalanceQuoteErr::Overflow)?;

    let post_inp = *inp_calc
        .sol_to_lst(req_inp_post_sol_value)
        .map_err(RebalanceQuoteErr::InpCalc)?
        .start();
    // TODO: this loop kinda suss for perf, improve
    let inp = (post_inp..)
        .map(|possible_post| {
            let inp_post_sol_value = *inp_calc
                .lst_to_sol(possible_post)
                .map_err(RebalanceQuoteErr::InpCalc)?
                .start();
            if inp_post_sol_value >= req_inp_post_sol_value {
                Ok(Some(
                    possible_post
                        .checked_sub(inp_reserves)
                        .ok_or(RebalanceQuoteErr::Overflow)?,
                ))
            } else {
                // not big enough, skip
                Ok(None)
            }
        })
        .filter_map(|r| r.transpose())
        .next()
        .map_or_else(|| Err(RebalanceQuoteErr::Overflow), |r| r)?;

    Ok(RebalanceQuote {
        out: amt,
        inp,
        out_mint,
        inp_mint,
    })
}

// TODO: exact in?
