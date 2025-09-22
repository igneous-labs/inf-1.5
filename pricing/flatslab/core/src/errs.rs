use core::{error::Error, fmt::Display};

use crate::{pricing::FlatSlabPricingErr, typedefs::MintNotFoundErr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FlatSlabProgramErr {
    CantRemoveLpMint,
    MintNotFound(MintNotFoundErr),
    MissingAdminSignature,
    Pricing(FlatSlabPricingErr),
    WrongSlabAcc,
}

impl Display for FlatSlabProgramErr {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::CantRemoveLpMint => f.write_str("CantRemoveLpMint"),
            Self::MintNotFound(e) => Display::fmt(&e, f),
            Self::MissingAdminSignature => f.write_str("MissingAdminSignature"),
            Self::Pricing(e) => Display::fmt(&e, f),
            Self::WrongSlabAcc => f.write_str("WrongSlabAcc"),
        }
    }
}

impl Error for FlatSlabProgramErr {}
