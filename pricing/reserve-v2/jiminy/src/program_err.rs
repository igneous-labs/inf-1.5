use core::fmt::Write as _;

use crate::log_buf::LogBuf;
use inf1_pp_reserve_v2_core::errs::ReserveV2ProgramErr;
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
/// pub const fn rv2pe_to_u32(e: ReserveV2ProgramErr) -> u32 {
///     use ReserveV2ProgramErr::*;
///     match e {
///         MintNotFound(_) => 0,
///         Pricing(_) => 1,
///     }
/// }
/// ```
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
        pub const fn rv2pe_to_u32(e: ReserveV2ProgramErr) -> u32 {
            use ReserveV2ProgramErr::*;
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
    CantRemoveRequiredMint,
    FeeNanosOutOfRange(_),
    MathOverflow,
    MintNotFound(_),
    NegativeBandDelta,
    OverCap(_),
    SameMint(_),
    ThresholdNanosOutOfRange(_),
    UnsupportedDeprecatedInstruction,
    WsolBalanceGtPoolSolValue(_),
    ZeroRetainedValue,
    ZeroPoolSolValue,
);

pub struct CustomProgErr(pub ReserveV2ProgramErr);

impl From<ReserveV2ProgramErr> for CustomProgErr {
    #[inline]
    fn from(e: ReserveV2ProgramErr) -> Self {
        Self(e)
    }
}

impl From<CustomProgErr> for ProgramError {
    /// Also `sol_log` logs the error string. Formatting is done into a
    /// fixed-size stack buffer, so this does not allocate.
    #[inline(never)]
    fn from(CustomProgErr(e): CustomProgErr) -> Self {
        let mut buf = LogBuf::<ERR_LOG_CAP>::new();
        let _ = write!(buf, "{e}");
        sol_log(buf.as_str());
        ProgramError::custom(rv2pe_to_u32(e))
    }
}

#[cfg(test)]
mod tests {
    use inf1_pp_reserve_v2_core::{
        errs::{OverCapErr, ReserveV2ProgramErr, SameMintErr, WsolBalanceGtPoolSolValueErr},
        typedefs::{FeeNanosOutOfRangeErr, MintNotFoundErr, ThresholdNanosOutOfRangeErr},
    };

    use super::ERR_LOG_CAP;

    /// Drift guard: every error message must fit in [`ERR_LOG_CAP`] so the
    /// `From` impl never truncates what it logs. Worst case is always at max
    /// field values.
    #[test]
    fn err_msgs_fit_log_cap() {
        use ReserveV2ProgramErr::*;
        let cases = [
            CantRemoveRequiredMint,
            FeeNanosOutOfRange(FeeNanosOutOfRangeErr { actual: u32::MAX }),
            MathOverflow,
            MintNotFound(MintNotFoundErr {
                expected_i: usize::MAX,
                mint: [0xff; 32],
            }),
            NegativeBandDelta,
            OverCap(OverCapErr {
                requested_out_sol_value: u64::MAX,
                wsol_balance: u64::MAX,
            }),
            SameMint(SameMintErr { mint: [0xff; 32] }),
            ThresholdNanosOutOfRange(ThresholdNanosOutOfRangeErr { actual: u32::MAX }),
            UnsupportedDeprecatedInstruction,
            WsolBalanceGtPoolSolValue(WsolBalanceGtPoolSolValueErr {
                pool_sol_value: u64::MAX,
                wsol_balance: u64::MAX,
            }),
            ZeroRetainedValue,
            ZeroPoolSolValue,
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
