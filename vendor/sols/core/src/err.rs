use core::convert::Infallible;
use core::error::Error;
use core::fmt::{Debug, Display, Formatter};
use core::ops::RangeInclusive;

use generic_array_struct::generic_array_struct;

use crate::typedefs::{Nanos, NanosOutOfRangeErr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SanctumSolsErr {
    HoldingNotEmpty,
    InitMintHasFreezeAuth,
    InitMintWrongDecimals,
    InvalidPda(InvalidPdaErr<'static>),
    Math,
    MintNotEmpty,
    NoSucceedingEndRebal,
    NotEnoughLiquidity(OutOfRangeErrU64),
    NotPortfolioHolding(SortedListNoAddrErr),
    NotWhitelistedSvc(SortedListNoAddrErr),
    OutOfRangeU32(OutOfRangeErrU32),
    OutstandingNotEmpty,
    PoolNotRebalancing,
    PoolRebalancing,
    PortfolioNotEmpty,
    ProtocolFeeNotEmpty,
    RebalExceedLimit(OutOfRangeErrU64),
    SelfSwap,
    SlippageToleranceExceeded(OutOfRangeErrU64),
}

impl Display for SanctumSolsErr {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        use SanctumSolsErr::*;

        match self {
            HoldingNotEmpty => f.write_str("holding must have 0 outstanding"),
            InitMintHasFreezeAuth => f.write_str("mint must not have freeze auth"),
            InitMintWrongDecimals => f.write_str("mint must have 9 d.p."),
            InvalidPda(e) => Display::fmt(&e, f),
            Math => f.write_str("math err"),
            MintNotEmpty => f.write_str("mint must have 0 supply"),
            NoSucceedingEndRebal => f.write_str("no succeeding EndRebal instruction"),
            NotEnoughLiquidity(e) => f.write_fmt(format_args!(
                "Not enough liquidity. Required: {}. Available: {}",
                e.actual(),
                e.end_incl()
            )),
            NotPortfolioHolding(e) => f.write_fmt(format_args!("portfolio {e}")),
            NotWhitelistedSvc(e) => f.write_fmt(format_args!("svc whitelist {e}")),
            OutOfRangeU32(e) => Display::fmt(&e, f),
            OutstandingNotEmpty => f.write_str("outstanding lamports must be 0"),
            PoolNotRebalancing => f.write_str("pool not currently rebalancing"),
            PoolRebalancing => f.write_str("pool currently rebalancing"),
            PortfolioNotEmpty => f.write_str("portfolio must be empty"),
            ProtocolFeeNotEmpty => f.write_str("protocol fee must be 0"),
            RebalExceedLimit(e) => f.write_fmt(format_args!(
                "rebalance exceeds limit ({} > {})",
                e.actual(),
                e.end_incl()
            )),
            SelfSwap => f.write_str("self swaps disallowed"),
            SlippageToleranceExceeded(e) => {
                f.write_str("slippage tolerance exceeded: ")?;
                if e.actual() < e.start_incl() {
                    f.write_fmt(format_args!("{} < {}", e.actual(), e.start_incl()))
                } else {
                    f.write_fmt(format_args!("{} > {}", e.actual(), e.end_incl()))
                }
            }
        }
    }
}

impl Error for SanctumSolsErr {}

impl From<Infallible> for SanctumSolsErr {
    #[inline]
    fn from(_: Infallible) -> Self {
        unreachable!()
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct InvalidPdaErr<'a> {
    pub name: &'a str,
}

pub type InvalidKnownPdaErr = InvalidPdaErr<'static>;

impl InvalidKnownPdaErr {
    pub const POOL: Self = Self { name: "pool" };
    pub const PORTFOLIO: Self = Self { name: "portfolio" };
    pub const PROTOCOL: Self = Self { name: "protocol" };
    pub const WSOL_BRIDGE: Self = Self {
        name: "wsol_bridge",
    };
    pub const POOL_ATA: Self = Self { name: "pool_ata" };
}

impl Display for InvalidPdaErr<'_> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("invalid {} PDA", self.name))
    }
}

impl Error for InvalidPdaErr<'_> {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SortedListNoAddrErr {
    pub addr: [u8; 32],

    /// The binary seach index that this addr would be inserted at
    /// if it was part of the sorted list, or usize::MAX if not applicable
    pub idx: usize,
}

impl Display for SortedListNoAddrErr {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("addr {:?} not in list", self.addr))
    }
}

impl Error for SortedListNoAddrErr {}

#[generic_array_struct(all pub)]
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OutOfRangeErr<T> {
    pub actual: T,
    pub start_incl: T,
    pub end_incl: T,
}

impl<T: Copy> OutOfRangeErr<T> {
    #[inline]
    pub const fn new(actual: T, range: RangeInclusive<T>) -> Self {
        Self::const_from_destr(OutOfRangeErrDestr {
            actual,
            start_incl: *range.start(),
            end_incl: *range.end(),
        })
    }
}

impl<T: Display> Display for OutOfRangeErr<T> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!(
            "{} out of range {}..={}",
            self.actual(),
            self.start_incl(),
            self.end_incl()
        ))
    }
}

impl<T: Debug + Display> Error for OutOfRangeErr<T> {}

pub type OutOfRangeErrU32 = OutOfRangeErr<u32>;

impl OutOfRangeErrU32 {
    #[inline]
    pub const fn from_nanos_out_of_range_err(
        NanosOutOfRangeErr { actual }: NanosOutOfRangeErr,
    ) -> Self {
        Self::new(actual, 0..=Nanos::DENOM)
    }
}

impl From<NanosOutOfRangeErr> for OutOfRangeErrU32 {
    #[inline]
    fn from(e: NanosOutOfRangeErr) -> Self {
        Self::from_nanos_out_of_range_err(e)
    }
}

pub type OutOfRangeErrU64 = OutOfRangeErr<u64>;

impl OutOfRangeErrU64 {
    #[inline]
    pub const fn actual_exceed_max(actual: u64, max_incl: u64) -> Self {
        Self::new(actual, 0..=max_incl)
    }

    #[inline]
    pub const fn actual_below_min(actual: u64, min_incl: u64) -> Self {
        Self::new(actual, min_incl..=u64::MAX)
    }
}
