use core::{error::Error, fmt::Display};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FlatFeePricingErr {
    Ratio,
}

impl Display for FlatFeePricingErr {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::Ratio => "ratio math error",
        })
    }
}

impl Error for FlatFeePricingErr {}
