use core::{error::Error, fmt::Display};

use crate::{pricing::FlatSlabPricingErr, typedefs::MintNotFoundErr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FlatSlabProgramErr {
    MintNotFound(MintNotFoundErr),
    Pricing(FlatSlabPricingErr),
    WrongSlabAcc,
    // TODO: add more variants as needed
}

impl Display for FlatSlabProgramErr {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::MintNotFound(e) => Display::fmt(&e, f),
            Self::Pricing(e) => Display::fmt(&e, f),
            Self::WrongSlabAcc => f.write_str("WrongSlabAcc"),
        }
    }
}

impl Error for FlatSlabProgramErr {}
