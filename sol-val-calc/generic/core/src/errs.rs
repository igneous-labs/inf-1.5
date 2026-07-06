use core::{error::Error, fmt::Display};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GenSvcErr {
    // Original errors copied from
    // https://github.com/igneous-labs/S/blob/66de438b9e049aacc193b5795d26ac055d86d770/idl/sol-value-calculator-programs/generic_pool_calculator.json#L204-L245
    UnexpectedProgramUpgrade,
    WrongPoolAccountType,
    StateAlreadyInitialized,
    WrongPoolProgram,
    WrongCalculatorStatePDA,
    InvalidCalculatorStateData,
    InvalidStakePoolProgramData,
    MathError,
}

impl Display for GenSvcErr {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::UnexpectedProgramUpgrade => {
                f.write_str("stake pool program has been updated since last UpdateLastUpgradeSlot")
            }
            Self::WrongPoolAccountType => f.write_str("stake pool account type is wrong"),
            Self::StateAlreadyInitialized => f.write_str("state already initialized"),
            Self::WrongPoolProgram => {
                f.write_str("calculator program is not for the given stake pool program")
            }
            Self::WrongCalculatorStatePDA => f.write_str("address of CalculatorState PDA is wrong"),
            Self::InvalidCalculatorStateData => f.write_str("invalid calculator state data"),
            Self::InvalidStakePoolProgramData => f.write_str("invalid stake pool program data"),
            Self::MathError => f.write_str("math error"),
        }
    }
}

impl Error for GenSvcErr {}

/// Example-usage:
///
/// ```ignore
/// errcode!(@ctr 1000; UnexpectedProgramUpgrade, Pricing(_));
/// ```
///
/// Generates:
///
/// ```ignore
/// pub const fn gen_svc_err_to_u32(e: GenSvcErr) -> u32 {
///     use GenSvcErr::*;
///     match e {
///         UnexpectedProgramUpgrade(_) => 1000,
///         Pricing(_) => 1001,
///     }
/// }
/// ```
///
/// TODO: also generate the oppposite u32 -> Option<Inf1CtlErr> conversion
/// for clients if required
macro_rules! errcode {
    // recursive-case
    (
        @ctr $ctr:expr;
        @match_inner { $($match_inner:tt)* };
        $variant:pat
        $(, $($tail:tt)*)?
    ) => {
        errcode!(
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
        pub const fn gen_svc_err_to_u32(e: GenSvcErr) -> u32 {
            use GenSvcErr::*;
            match e {
                $($match_inner)*
            }
        }
    };
    () => {};

    // start
    (@ctr $ctr:expr; $($tail:tt)*) => { errcode!(@ctr $ctr; @match_inner {}; $($tail)*); };
}

errcode!(
    @ctr 1000;
    UnexpectedProgramUpgrade,
    WrongPoolAccountType,
    StateAlreadyInitialized,
    WrongPoolProgram,
    WrongCalculatorStatePDA,
    InvalidCalculatorStateData,
    InvalidStakePoolProgramData,
    MathError
);

impl From<GenSvcErr> for u32 {
    #[inline]
    fn from(value: GenSvcErr) -> Self {
        gen_svc_err_to_u32(value)
    }
}
