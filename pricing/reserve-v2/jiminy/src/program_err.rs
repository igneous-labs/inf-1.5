use inf1_pp_reserve_v2_core::errs::ReserveV2ProgramErr;
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
    MathOverflow,
    MintNotFound(_),
    OverCap(_),
    SameMint(_),
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
    /// Also `sol_log` logs the error string.
    #[inline]
    fn from(CustomProgErr(e): CustomProgErr) -> Self {
        let msg = e.to_string();
        sol_log(&msg);
        ProgramError::custom(rv2pe_to_u32(e))
    }
}
