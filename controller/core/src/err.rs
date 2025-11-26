use core::{error::Error, fmt::Display};

use crate::typedefs::{
    fee_nanos::FeeNanosTooLargeErr, rps::RpsTooSmallErr, uq0f63::UQ0F63TooLargeErr,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Inf1CtlErr {
    // Original errors copied from
    // https://github.com/igneous-labs/S/blob/master/generated/s_controller_interface/src/errors.rs#L7-L87
    InvalidPoolStateData,
    InvalidLstStateListData,
    InvalidDisablePoolAuthorityListData,
    InvalidRebalanceRecordData,
    MathError,
    PoolRebalancing,
    PoolDisabled,
    PoolEnabled,
    InvalidLstIndex,
    InvalidReserves,
    IncorrectSolValueCalculator,
    FaultySolValueCalculator,
    IncorrectLstStateList,
    IncorrectPoolState,
    LstInputDisabled,
    NoSucceedingEndRebalance,
    IncorrectRebalanceRecord,
    PoolNotRebalancing,
    PoolWouldLoseSolValue,
    LstStillHasValue,
    IncorrectPricingProgram,
    SlippageToleranceExceeded,
    NotEnoughLiquidity,
    IndexTooLarge,
    InvalidDisablePoolAuthorityIndex,
    UnauthorizedDisablePoolAuthoritySigner,
    InvalidDisablePoolAuthority,
    UnauthorizedSetRebalanceAuthoritySigner,
    IncorrectDisablePoolAuthorityList,
    FeeTooHigh,
    NotEnoughFees,
    ZeroValue,
    FaultyPricingProgram,
    IncorrectLpMintInitialization,
    DuplicateLst,
    SwapSameLst,
    DuplicateDisablePoolAuthority,

    // v2 additions
    WrongPoolStateVers(WrongVersErr),
    RpsOob(RpsOobErr),
    FeeNanosOob(FeeNanosTooLargeErr),
}

impl Display for Inf1CtlErr {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use Inf1CtlErr::*;

        match self {
            InvalidPoolStateData
            | InvalidLstStateListData
            | InvalidDisablePoolAuthorityListData
            | InvalidRebalanceRecordData
            | MathError
            | PoolRebalancing
            | PoolDisabled
            | PoolEnabled
            | InvalidLstIndex
            | InvalidReserves
            | IncorrectSolValueCalculator
            | FaultySolValueCalculator
            | IncorrectLstStateList
            | IncorrectPoolState
            | LstInputDisabled
            | NoSucceedingEndRebalance
            | IncorrectRebalanceRecord
            | PoolNotRebalancing
            | PoolWouldLoseSolValue
            | LstStillHasValue
            | IncorrectPricingProgram
            | SlippageToleranceExceeded
            | NotEnoughLiquidity
            | IndexTooLarge
            | InvalidDisablePoolAuthorityIndex
            | UnauthorizedDisablePoolAuthoritySigner
            | InvalidDisablePoolAuthority
            | UnauthorizedSetRebalanceAuthoritySigner
            | IncorrectDisablePoolAuthorityList
            | FeeTooHigh
            | NotEnoughFees
            | ZeroValue
            | FaultyPricingProgram
            | IncorrectLpMintInitialization
            | DuplicateLst
            | SwapSameLst
            | DuplicateDisablePoolAuthority => core::fmt::Debug::fmt(self, f),
            WrongPoolStateVers(e) => f.write_fmt(format_args!("WrongPoolStateVers. {e}")),
            RpsOob(e) => f.write_fmt(format_args!("RpsOob. {e}")),
            FeeNanosOob(e) => f.write_fmt(format_args!("FeeNanosOob. {e}")),
        }
    }
}

impl Error for Inf1CtlErr {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WrongVersErr {
    pub expected: u8,
    pub actual: u8,
}

impl Display for WrongVersErr {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let Self { expected, actual } = self;
        f.write_fmt(format_args!(
            "Expected vers: {expected}. Actual vers: {actual}"
        ))
    }
}

impl Error for WrongVersErr {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RpsOobErr {
    Rps(RpsTooSmallErr),
    UQ0F63(UQ0F63TooLargeErr),
}

impl Display for RpsOobErr {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Rps(e) => e.fmt(f),
            Self::UQ0F63(e) => e.fmt(f),
        }
    }
}

impl Error for RpsOobErr {}
