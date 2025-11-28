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

    /// The amount of fee accrued to the pool,
    /// in terms of sol value (lamports)
    pub fee: u64,

    /// This is INF for RemoveLiquidity
    pub inp_mint: [u8; 32],

    /// This is INF for AddLiquidity
    pub out_mint: [u8; 32],
}
