use inf1_ctl_core::err::Inf1CtlErr;
use jiminy_log::sol_log;
use jiminy_program_error::ProgramError;

/// Example-usage:
///
/// ```ignore
/// seqerr!(MintNotFound(_), Pricing(_));
/// ```
///
/// Generates:
///
/// ```ignore
/// pub const fn inf1_ctl_err_to_u32(e: Inf1CtlErr) -> u32 {
///     use Inf1CtlErr::*;
///     match e {
///         MintNotFound(_) => 0,
///         Pricing(_) => 1,
///     }
/// }
/// ```
///
/// TODO: also generate the oppposite u32 -> Option<Inf1CtlErr> conversion
/// for clients if required
macro_rules! seqerr {
    // recursive-case
    (
        @ctr $ctr:expr;
        @match_inner { $($match_inner:tt)* };
        $variant:pat
        $(, $($tail:tt)*)?
    ) => {
        seqerr!(
            @ctr ($ctr + 1);
            @match_inner {
                $variant => $ctr,
                $($match_inner)*
            };
            $($($tail)*)?
        );
    };

    // base-cases
    (
        @ctr $ctr:expr;
        @match_inner { $($match_inner:tt)* };
    ) => {
        pub const fn inf1_ctl_err_to_u32(e: Inf1CtlErr) -> u32 {
            use Inf1CtlErr::*;
            match e {
                $($match_inner)*
            }
        }
    };
    () => {};

    // start
    ($($tail:tt)*) => { seqerr!(@ctr 0; @match_inner {}; $($tail)*); };
}

seqerr!(
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
    InvalidProtocolFeeAccumulator,
);

pub struct Inf1CtlCustomProgErr(pub Inf1CtlErr);

impl From<Inf1CtlCustomProgErr> for ProgramError {
    // Note: to_string() + log adds around 15kb to binsize
    /// Also `sol_msg` logs the error string.
    #[inline]
    fn from(Inf1CtlCustomProgErr(e): Inf1CtlCustomProgErr) -> Self {
        let msg = e.to_string();
        sol_log(&msg);
        ProgramError::custom(inf1_ctl_err_to_u32(e))
    }
}
