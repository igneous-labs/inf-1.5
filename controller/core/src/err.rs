//! Copied from https://github.com/igneous-labs/S/blob/master/generated/s_controller_interface/src/errors.rs#L7-L87

use core::{error::Error, fmt::Display};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Inf1CtlErr {
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
    MissingRequiredSignature,
}

impl Display for Inf1CtlErr {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(self, f)
    }
}

impl Error for Inf1CtlErr {}
