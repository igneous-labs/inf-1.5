use core::{error::Error, fmt::Display};

use crate::typedefs::MintNotFoundErr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReserveV2ProgramErr {
    MathOverflow,
    MintNotFound(MintNotFoundErr),
    OverCap(OverCapErr),
    SameMint(SameMintErr),
    UnsupportedDeprecatedInstruction,
    WsolBalanceGtPoolSolValue(WsolBalanceGtPoolSolValueErr),
    ZeroRetainedValue,
    ZeroPoolSolValue,
}

impl Display for ReserveV2ProgramErr {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::MathOverflow => f.write_str("MathOverflow"),
            Self::MintNotFound(e) => Display::fmt(e, f),
            Self::OverCap(e) => Display::fmt(e, f),
            Self::SameMint(e) => Display::fmt(e, f),
            Self::UnsupportedDeprecatedInstruction => {
                f.write_str("UnsupportedDeprecatedInstruction")
            }
            Self::WsolBalanceGtPoolSolValue(e) => Display::fmt(e, f),
            Self::ZeroRetainedValue => f.write_str("ZeroRetainedValue"),
            Self::ZeroPoolSolValue => f.write_str("ZeroPoolSolValue"),
        }
    }
}

impl Error for ReserveV2ProgramErr {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SameMintErr {
    pub mint: [u8; 32],
}

impl Display for SameMintErr {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("SameMint")
    }
}

impl Error for SameMintErr {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OverCapErr {
    pub requested_out_sol_value: u64,
    pub wsol_balance: u64,
}

impl Display for OverCapErr {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let Self {
            requested_out_sol_value,
            wsol_balance,
        } = self;
        f.write_fmt(format_args!(
            "requested output SOL value {requested_out_sol_value} > wSOL balance {wsol_balance}"
        ))
    }
}

impl Error for OverCapErr {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WsolBalanceGtPoolSolValueErr {
    pub pool_sol_value: u64,
    pub wsol_balance: u64,
}

impl Display for WsolBalanceGtPoolSolValueErr {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let Self {
            pool_sol_value,
            wsol_balance,
        } = self;
        f.write_fmt(format_args!(
            "wSOL balance {wsol_balance} > pool SOL value {pool_sol_value}"
        ))
    }
}

impl Error for WsolBalanceGtPoolSolValueErr {}
