use core::fmt::Write as _;

use crate::log_buf::LogBuf;
use inf1_ctl_core::err::Inf1CtlErr;
use jiminy_log::sol_log;
use jiminy_program_error::ProgramError;

/// Max bytes of any error message formatted by the `From` impl below.
///
/// Longer messages are silently truncated; guarded by
/// `tests::err_msgs_fit_log_cap`.
const ERR_LOG_CAP: usize = 96;

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
    WrongPoolStateVers(_),
    InvalidPoolStateDataV2(_),
    TimeWentBackwards,
    UnauthorizedSetRpsAuthoritySigner,
);

pub struct Inf1CtlCustomProgErr(pub Inf1CtlErr);

impl From<Inf1CtlErr> for Inf1CtlCustomProgErr {
    #[inline]
    fn from(e: Inf1CtlErr) -> Self {
        Self(e)
    }
}

impl From<Inf1CtlCustomProgErr> for ProgramError {
    /// Also `sol_log` logs the error string. Formatting is done into a
    /// fixed-size stack buffer, so this does not allocate.
    #[inline(never)]
    fn from(Inf1CtlCustomProgErr(e): Inf1CtlCustomProgErr) -> Self {
        let mut buf = LogBuf::<ERR_LOG_CAP>::new();
        let _ = write!(buf, "{e}");
        sol_log(buf.as_str());
        ProgramError::custom(inf1_ctl_err_to_u32(e))
    }
}

#[cfg(test)]
mod tests {
    use inf1_ctl_core::{
        err::{Inf1CtlErr, InvalidPoolStateDataErrV2, RpsOobErr, WrongVersErr},
        typedefs::{
            fee_nanos::FeeNanosTooLargeErr,
            rps::RpsTooSmallErr,
            uq0f63::{UQ0F63TooLargeErr, UQ0F63},
        },
    };

    use super::ERR_LOG_CAP;

    /// Drift guard: every error message must fit in [`ERR_LOG_CAP`] so the
    /// `From` impl never truncates what it logs. Worst case is always at max
    /// field values.
    #[test]
    fn err_msgs_fit_log_cap() {
        use Inf1CtlErr::*;
        let cases = [
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
            TimeWentBackwards,
            UnauthorizedSetRpsAuthoritySigner,
            WrongPoolStateVers(WrongVersErr {
                expected: u8::MAX,
                actual: u8::MAX,
            }),
            InvalidPoolStateDataV2(InvalidPoolStateDataErrV2::Rps(RpsOobErr::Rps(
                RpsTooSmallErr {
                    actual: UQ0F63::new(0).unwrap(),
                },
            ))),
            InvalidPoolStateDataV2(InvalidPoolStateDataErrV2::Rps(RpsOobErr::UQ0F63(
                UQ0F63TooLargeErr { actual: u64::MAX },
            ))),
            InvalidPoolStateDataV2(InvalidPoolStateDataErrV2::ProtocolFeeNanos(
                FeeNanosTooLargeErr { actual: u32::MAX },
            )),
        ];
        for e in cases {
            let s = e.to_string();
            assert!(
                s.len() <= ERR_LOG_CAP,
                "msg too long ({} > {ERR_LOG_CAP}): {s}",
                s.len()
            );
        }
    }
}
