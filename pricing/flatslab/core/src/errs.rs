use core::{error::Error, fmt::Display};

use crate::pricing::FlatSlabPricingErr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FlatSlabProgramErr {
    Pricing(FlatSlabPricingErr),
    // TODO: add more variants as needed
}

impl Display for FlatSlabProgramErr {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Pricing(e) => Display::fmt(&e, f),
        }
    }
}

impl Error for FlatSlabProgramErr {}
