use crate::{err::NotEnoughLiquidityErr, quote::Quote};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RemoveLiqQuoteArgs<O, P> {
    pub amt: u64,

    pub lp_token_supply: u64,

    pub pool_total_sol_value: u64,

    pub out_reserves: u64,

    /// Read from PoolState
    pub lp_protocol_fee_bps: u16,

    pub inp_mint: [u8; 32],

    pub lp_mint: [u8; 32],

    pub inp_calc: O,

    pub pricing: P,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct RemoveLiqQuote(pub Quote);

impl RemoveLiqQuote {
    #[inline]
    pub const fn fee_mint(&self) -> &[u8; 32] {
        &self.0.out_mint
    }
}

pub type RemoveLiqQuoteResult<O, P> = Result<RemoveLiqQuote, RemoveLiqQuoteErr<O, P>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RemoveLiqQuoteErr<O, P> {
    NotEnougLiquidity(NotEnoughLiquidityErr),
    OutCalc(O),
    Overflow,
    Pricing(P),
    ZeroValue,
}
