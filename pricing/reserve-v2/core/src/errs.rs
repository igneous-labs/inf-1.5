use core::{error::Error, fmt::Display};

use crate::typedefs::{InvalidFeeEntryErr, MintNotFoundErr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReserveV2ProgramErr {
    InvalidFeeEntry(InvalidFeeEntryErr),
    MathOverflow,
    MintNotFound(MintNotFoundErr),
    OverDrain(OverDrainErr),
    SameMint(SameMintErr),
    UnsupportedDeprecatedInstruction,
    ZeroRetainedValue,
    ZeroPoolSolValue,
}

impl Display for ReserveV2ProgramErr {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidFeeEntry(e) => Display::fmt(e, f),
            Self::MathOverflow => f.write_str("MathOverflow"),
            Self::MintNotFound(e) => Display::fmt(e, f),
            Self::OverDrain(e) => Display::fmt(e, f),
            Self::SameMint(e) => Display::fmt(e, f),
            Self::UnsupportedDeprecatedInstruction => {
                f.write_str("UnsupportedDeprecatedInstruction")
            }
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
pub struct OverDrainErr {
    pub requested_wsol_out: u64,
    pub wsol_balance: u64,
}

impl Display for OverDrainErr {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let Self {
            requested_wsol_out,
            wsol_balance,
        } = self;
        f.write_fmt(format_args!(
            "requested wSOL out {requested_wsol_out} > wSOL balance {wsol_balance}"
        ))
    }
}

impl Error for OverDrainErr {}
