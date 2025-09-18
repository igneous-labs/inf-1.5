pub mod liquidity;
pub mod rebalance;
pub mod swap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Quote {
    /// Amount of input tokens given by the user to the pool,
    /// after fees. This is exactly the amount of tokens that
    /// will leave the user's wallet.
    pub inp: u64,

    /// Amount of output tokens returned by the pool to the user,
    /// after fees. This is exactly the amount of tokens that
    /// will enter the user's wallet.
    pub out: u64,

    /// The amount of fee accrued to pool LPs,
    /// accumulated in the pool reserves.
    ///
    /// Which mint it is denoted in (whether inp_mint or out_mint)
    /// depends on the newtype this struct is wrapped in
    pub lp_fee: u64,

    /// The amount of fee accrued to the protocol,
    /// to be transferred to the protocol fee accumulator account.
    ///
    /// Which mint it is denoted in (whether inp_mint or out_mint)
    /// depends on the newtype this struct is wrapped in,
    /// but it will always be the same mint as that of `lp_fee`
    pub protocol_fee: u64,

    /// This is INF for RemoveLiquidity
    pub inp_mint: [u8; 32],

    /// This is INF for AddLiquidity
    pub out_mint: [u8; 32],
}
