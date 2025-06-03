use err::SwapQuoteErr;
use sanctum_fee_ratio::{
    ratio::{Ceil, Ratio},
    Fee,
};

use super::Quote;

pub mod err;
pub mod exact_in;
pub mod exact_out;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SwapQuoteArgs<I, O, P> {
    pub amt: u64,

    /// Token balance of the pool's output LST reserves
    pub out_reserves: u64,

    /// Read from PoolState
    pub trading_protocol_fee_bps: u16,

    pub inp_mint: [u8; 32],

    pub out_mint: [u8; 32],

    pub inp_calc: I,

    pub out_calc: O,

    pub pricing: P,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct SwapQuote(pub Quote);

impl SwapQuote {
    #[inline]
    pub const fn fee_mint(&self) -> &[u8; 32] {
        &self.0.out_mint
    }
}

pub type SwapQuoteResult<I, O, P> = Result<SwapQuote, SwapQuoteErr<I, O, P>>;

type Tpf = Fee<Ceil<Ratio<u16, u16>>>;

pub const fn trading_protocol_fee(trading_protocol_fee_bps: u16) -> Option<Tpf> {
    Tpf::new(Ratio {
        n: trading_protocol_fee_bps,
        d: 10_000,
    })
}
