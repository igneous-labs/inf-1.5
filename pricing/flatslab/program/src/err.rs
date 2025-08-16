use inf1_pp_flatslab_core::errs::FlatSlabProgramErr;
use jiminy_entrypoint::program_error::ProgramError;

/// Example-usage:
///
/// ```ignore
/// seqerr!(MintNotFound(_), Pricing(_));
/// ```
///
/// Generates:
///
/// ```ignore
/// pub const fn fspe_to_u32(e: FlatSlabProgramErr) -> u32 {
///     use FlatSlabProgramErr::*;
///     match e {
///         MintNotFound(_) => 1,
///         Pricing(_) => 2,
///     }
/// }
/// ```
///
/// NB: we start at 1 instead of 0 to avoid Custom(0)'s special-case handling
/// (not a pure bitshift to convert to NonZeroU64)
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
        pub const fn fspe_to_u32(e: FlatSlabProgramErr) -> u32 {
            use FlatSlabProgramErr::*;
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
    MintNotFound(_),
    MissingAdminSignature,
    Pricing(_),
    WrongSlabAcc
);

pub struct CustomProgErr(pub FlatSlabProgramErr);

impl From<CustomProgErr> for ProgramError {
    fn from(CustomProgErr(e): CustomProgErr) -> Self {
        ProgramError::custom(fspe_to_u32(e))
    }
}
