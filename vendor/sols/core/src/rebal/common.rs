use core::{convert::Infallible, error::Error, fmt::Display};

use generic_array_struct::generic_array_struct;
use sanctum_u64_ratio::Ceil;

use crate::{
    accounts::PoolV1LamportVals,
    err::{OutOfRangeErrU64, SanctumSolsErr},
    typedefs::Nanos,
    utils::InpOut,
};

/// For checking the pool's SOL reserve ratio
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RRArgs {
    /// rrr_floor in sol_inp case
    ///
    /// rrr_ceil in sol_out case
    pub rrr_limit: Nanos,

    pub pool_lamports: PoolV1LamportVals,

    pub pool_acc_lamports: u64,
}

#[generic_array_struct(all pub)]
#[repr(transparent)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RebalQtys<T> {
    /// Rebalance token amounts.
    ///
    /// Amounts include protocol fees
    pub amt: T,

    /// SOL values of [`Self::amt`]
    ///
    /// Might not be eq if
    /// - Rebalancing from SOL due to SOL out loss tolerance
    /// - Rebalancing to SOL, because the user/rebalancer will be entitled
    ///   to more SOL value out than they need to put in, and also because `amt.out` will be
    ///   less protocol fees
    pub sol_value: T,
}

pub type RebalInpOut = RebalQtys<InpOut<u64>>;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RebalQuote {
    pub qtys: RebalInpOut,

    /// Change in outstanding
    pub outstanding: u64,
}

/// A rebalance quote with just the corresponding input and output SOL value amounts,
/// without the token amounts
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RebalQuoteSolVal {
    pub sol_value: InpOut<u64>,

    /// Change in outstanding
    pub outstanding: u64,
}

/// Verifies that the SOL rebalanced out does not cause SOL reserves
/// to drop below ceil of `rr_args.rrr_limit`;
/// only SOL in excess of ceil can be rebalanced out
///
/// ## Params
/// - `rr_args`
/// - `outstanding_inc` increase in outstanding SOL <-> SOL being rebalanced out
#[inline]
pub const fn verify_rebal_above_ceil(
    RRArgs {
        rrr_limit,
        pool_lamports,
        pool_acc_lamports,
    }: &RRArgs,
    outstanding_inc: u64,
) -> Result<(), QuoteRebalErr<Infallible>> {
    let dep_due = match pool_lamports.dep_due_checked(*pool_acc_lamports) {
        None => return Err(QuoteRebalErr::Math),
        Some(x) => x,
    };
    let liq_avail = match pool_lamports.liq_avail_checked(*pool_acc_lamports) {
        None => return Err(QuoteRebalErr::Math),
        Some(x) => x,
    };
    let min_liq_avail = match Ceil(rrr_limit.into_ratio()).apply(dep_due) {
        None => return Err(QuoteRebalErr::Math),
        Some(x) => x,
    };
    let max_liq_avail_dec = liq_avail.saturating_sub(min_liq_avail);
    if outstanding_inc > max_liq_avail_dec {
        Err(QuoteRebalErr::RebalExceedLimit(
            OutOfRangeErrU64::actual_exceed_max(outstanding_inc, max_liq_avail_dec),
        ))
    } else {
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QuoteRebalErr<I> {
    InpSvc(I),
    Math,
    NotEnoughLiquidity(OutOfRangeErrU64),
    RebalExceedLimit(OutOfRangeErrU64),
}

impl<I> QuoteRebalErr<I> {
    #[inline]
    pub const fn conv_infallible(e: QuoteRebalErr<Infallible>) -> Self {
        match e {
            QuoteRebalErr::InpSvc(_infallible) => unreachable!(),
            QuoteRebalErr::Math => Self::Math,
            QuoteRebalErr::NotEnoughLiquidity(e) => Self::NotEnoughLiquidity(e),
            QuoteRebalErr::RebalExceedLimit(e) => Self::RebalExceedLimit(e),
        }
    }
}

impl<I: Into<SanctumSolsErr>> From<QuoteRebalErr<I>> for SanctumSolsErr {
    #[inline]
    fn from(v: QuoteRebalErr<I>) -> Self {
        match v {
            QuoteRebalErr::InpSvc(e) => e.into(),
            QuoteRebalErr::Math => SanctumSolsErr::Math,
            QuoteRebalErr::NotEnoughLiquidity(e) => SanctumSolsErr::NotEnoughLiquidity(e),
            QuoteRebalErr::RebalExceedLimit(e) => SanctumSolsErr::RebalExceedLimit(e),
        }
    }
}

impl<I: Into<SanctumSolsErr> + Copy> Display for QuoteRebalErr<I> {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Into::<SanctumSolsErr>::into(*self).fmt(f)
    }
}

impl<I: Into<SanctumSolsErr> + Copy + core::fmt::Debug> Error for QuoteRebalErr<I> {}
