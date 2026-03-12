use err::QuoteErr;

use super::Quote;

pub mod err;
pub mod exact_in;
pub mod exact_out;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct QuoteArgs<I, O, P> {
    pub amt: u64,

    /// Token balance of the pool's output LST reserves
    ///
    /// Set to u64::MAX if out_mint=INF (add liquidity)
    pub out_reserves: u64,

    pub inp_mint: [u8; 32],

    pub out_mint: [u8; 32],

    pub inp_calc: I,

    pub out_calc: O,

    pub pricing: P,
}

pub type QuoteResult<I, O, P> = Result<Quote, QuoteErr<I, O, P>>;
