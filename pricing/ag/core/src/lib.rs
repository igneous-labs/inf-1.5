#![cfg_attr(not(test), no_std)]

use core::{error::Error, fmt::Display};

// Re-exports
pub use inf1_pp_flatfee_core;
pub use inf1_pp_flatslab_core;

use crate::internal_utils::map_variant_pure;

pub mod instructions;
pub mod pricing;

mod internal_utils;

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
        map_variant_pure!(self, Iterator::next)
    }

    #[inline]
    fn fold<B, F>(self, init: B, f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        map_variant_pure!(self, (|p| Iterator::fold(p, init, f)))
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
        map_variant_pure!(self, AsRef::as_ref)
    }
}

// Display + Error blanket

impl<FlatFee: Error, FlatSlab: Error> Display for PricingAg<FlatFee, FlatSlab> {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        map_variant_pure!(self, (|p| Display::fmt(&p, f)))
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
