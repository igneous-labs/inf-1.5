#![cfg_attr(not(test), no_std)]

pub mod instructions;
pub mod pricing;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PricingAg<FlatFee> {
    FlatFee(FlatFee),
    // TODO: SimpFlatFee variant
}

impl<FlatFee> PricingAg<FlatFee> {
    #[inline]
    pub const fn ty(&self) -> PricingAgTy {
        match self {
            Self::FlatFee(_) => PricingAgTy::FlatFee,
        }
    }
}

// Iterator blanket
impl<T, FlatFee: Iterator<Item = T>> Iterator for PricingAg<FlatFee> {
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::FlatFee(p) => p.next(),
        }
    }

    #[inline]
    fn fold<B, F>(self, init: B, f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        match self {
            Self::FlatFee(p) => p.fold(init, f),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PricingAgTy {
    FlatFee,
}

impl PricingAgTy {
    #[inline]
    pub const fn program_id(&self) -> &[u8; 32] {
        match self {
            Self::FlatFee => &inf1_pp_flatfee_core::ID,
        }
    }

    #[inline]
    pub const fn try_from_program_id(program_id: &[u8; 32]) -> Option<Self> {
        Some(match *program_id {
            inf1_pp_flatfee_core::ID => Self::FlatFee,
            _ => return None,
        })
    }
}
