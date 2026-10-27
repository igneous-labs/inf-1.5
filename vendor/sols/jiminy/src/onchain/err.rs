//! A note about error conversions for optimal CUs and program binsize
//! - Changing return type from `BuiltInProgramError` to `ProgramError` for fns in general reduce CUs and binsize
//!   probably because `ProgramError` is a simpler repr than `BuiltInProgramError` (just a NonZeroU64)
//! - Changing return type from `SanctumSolProgrErr` to `ProgramError` for fns in general increase CUs and binsize
//!   probably because of the expensive conversion + logging logic that is inlined

use jiminy_log::{sol_log, sol_log_pubkey};
use jiminy_program_error::ProgramError;
use sanctum_sols_core::err::SanctumSolsErr;

/// Example-usage:
///
/// ```ignore
/// seqerr!(InvalidPda(_), NotEnoughLiquidity(_));
/// ```
///
/// Generates:
///
/// ```ignore
/// pub const fn sanctum_sols_err_to_u32(e: Inf1CtlErr) -> u32 {
///     use SanctumSolsErr::*;
///     match e {
///         InvalidPda(_) => 1,
///         NotEnoughLiquidity(_) => 2,
///     }
/// }
/// ```
///
/// Note we start from custom error 1 to avoid the more complex conversion case for custom 0
///
/// TODO: also generate the oppposite u32 -> Option<SanctumSolsErr> conversion
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
        pub const fn sanctum_sols_err_to_u32(e: SanctumSolsErr) -> u32 {
            use SanctumSolsErr::*;
            match e {
                $($match_inner)*
            }
        }
    };
    () => {};

    // start
    ($($tail:tt)*) => { seqerr!(@ctr 1; @match_inner {}; $($tail)*); };
}

seqerr!(
    HoldingNotEmpty,
    InitMintHasFreezeAuth,
    InitMintWrongDecimals,
    InvalidPda(_),
    Math,
    MintNotEmpty,
    NoSucceedingEndRebal,
    NotEnoughLiquidity(_),
    NotPortfolioHolding(_),
    NotWhitelistedSvc(_),
    OutOfRangeU32(_),
    OutstandingNotEmpty,
    PoolNotRebalancing,
    PoolRebalancing,
    PortfolioNotEmpty,
    ProtocolFeeNotEmpty,
    RebalExceedLimit(_),
    SelfSwap,
    SlippageToleranceExceeded(_),
);

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SanctumSolsProgErr(pub SanctumSolsErr);

impl From<SanctumSolsErr> for SanctumSolsProgErr {
    #[inline]
    fn from(value: SanctumSolsErr) -> Self {
        Self(value)
    }
}

impl From<SanctumSolsProgErr> for ProgramError {
    // Note: to_string() + log adds around 15kb to binsize
    /// Also `sol_msg` logs the error string, so that `?`
    /// in onchain programs logs an error message before conversion
    #[inline]
    fn from(SanctumSolsProgErr(e): SanctumSolsProgErr) -> Self {
        use SanctumSolsErr::*;

        match e {
            // errs that require special formatting
            // - includes pubkey(s)
            NotPortfolioHolding(e) => {
                sol_log_pubkey(&e.addr);
                sol_log("not in portfolio");
            }
            NotWhitelistedSvc(e) => {
                sol_log_pubkey(&e.addr);
                sol_log("not whitelisted SOL value calculator");
            }

            // errs that can just use their Display fmt
            e => {
                let msg = e.to_string();
                sol_log(&msg);
            }
        }

        ProgramError::custom(sanctum_sols_err_to_u32(e))
    }
}
