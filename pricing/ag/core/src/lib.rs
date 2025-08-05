#![cfg_attr(not(test), no_std)]

pub mod instructions;
pub mod pricing;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PricingAccsAg<FlatFee> {
    FlatFee(FlatFee),
    // TODO: SimpFlatFee variant
}

impl<A, FlatFee> AsRef<A> for PricingAccsAg<FlatFee>
where
    A: ?Sized,
    FlatFee: AsRef<A>,
{
    #[inline]
    fn as_ref(&self) -> &A {
        match self {
            Self::FlatFee(g) => g.as_ref(),
        }
    }
}

impl<FlatFee> PricingAccsAg<FlatFee> {
    #[inline]
    pub const fn ty(&self) -> PricingAccsAgTy {
        match self {
            Self::FlatFee(_) => PricingAccsAgTy::FlatFee,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PricingAccsAgTy {
    FlatFee,
}

impl PricingAccsAgTy {
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
