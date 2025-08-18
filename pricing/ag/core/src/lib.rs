#![cfg_attr(not(test), no_std)]

use core::{error::Error, fmt::Display};

// Re-exports
pub use inf1_pp_flatfee_core;

pub mod instructions;
pub mod pricing;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PricingAg<FlatFee, FlatSlab> {
    FlatFee(FlatFee),
    FlatSlab(FlatSlab),
}

impl<FlatFee, FlatSlab> PricingAg<FlatFee, FlatSlab> {
    #[inline]
    pub const fn ty(&self) -> PricingAgTy {
        match self {
            Self::FlatFee(_) => PricingAgTy::FlatFee(()),
            Self::FlatSlab(_) => PricingAgTy::FlatSlab(()),
        }
    }

    #[inline]
    pub const fn program_id(&self) -> &[u8; 32] {
        match self {
            Self::FlatFee(_) => &inf1_pp_flatfee_core::ID,
            Self::FlatSlab(_) => &inf1_pp_flatslab_core::ID,
        }
    }
}

// Iterator blanket
impl<T, FlatFee: Iterator<Item = T>, FlatSlab: Iterator<Item = T>> Iterator
    for PricingAg<FlatFee, FlatSlab>
{
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::FlatFee(p) => p.next(),
            Self::FlatSlab(p) => p.next(),
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
            Self::FlatSlab(p) => p.fold(init, f),
        }
    }
}

// AsRef blanket
impl<A, FlatFee, FlatSlab> AsRef<A> for PricingAg<FlatFee, FlatSlab>
where
    A: ?Sized,
    FlatFee: AsRef<A>,
    FlatSlab: AsRef<A>,
{
    #[inline]
    fn as_ref(&self) -> &A {
        match self {
            Self::FlatFee(g) => g.as_ref(),
            Self::FlatSlab(g) => g.as_ref(),
        }
    }
}

// Display + Error blanket

impl<FlatFee: Error, FlatSlab: Error> Display for PricingAg<FlatFee, FlatSlab> {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::FlatFee(e) => Display::fmt(&e, f),
            Self::FlatSlab(e) => Display::fmt(&e, f),
        }
    }
}

impl<FlatFee: Error, FlatSlab: Error> Error for PricingAg<FlatFee, FlatSlab> {}

pub type PricingAgTy = PricingAg<(), ()>;

impl PricingAgTy {
    #[inline]
    pub const fn try_from_program_id(program_id: &[u8; 32]) -> Option<Self> {
        Some(match *program_id {
            inf1_pp_flatfee_core::ID => Self::FlatFee(()),
            inf1_pp_flatslab_core::ID => Self::FlatSlab(()),
            _ => return None,
        })
    }
}
