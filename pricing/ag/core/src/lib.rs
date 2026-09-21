#![cfg_attr(not(test), no_std)]

use core::{error::Error, fmt::Display};

// Re-exports
pub use inf1_pp_flatfee_core;
pub use inf1_pp_flatslab_core;
pub use inf1_pp_reserve_v2_core;

use crate::internal_utils::map_variant_pure;

pub mod instructions;
pub mod pricing;

mod internal_utils;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PricingAg<FlatFee, FlatSlab, ReserveV2> {
    FlatFee(FlatFee),
    FlatSlab(FlatSlab),
    ReserveV2(ReserveV2),
}

impl<FlatFee, FlatSlab, ReserveV2> PricingAg<FlatFee, FlatSlab, ReserveV2> {
    #[inline]
    pub const fn ty(&self) -> PricingAgTy {
        match self {
            Self::FlatFee(_) => PricingAgTy::FlatFee(()),
            Self::FlatSlab(_) => PricingAgTy::FlatSlab(()),
            Self::ReserveV2(_) => PricingAgTy::ReserveV2(()),
        }
    }

    #[inline]
    pub const fn program_id(&self) -> &[u8; 32] {
        match self {
            Self::FlatFee(_) => &inf1_pp_flatfee_core::ID,
            Self::FlatSlab(_) => &inf1_pp_flatslab_core::ID,
            Self::ReserveV2(_) => &inf1_pp_reserve_v2_core::ID,
        }
    }
}

// Iterator blanket
impl<
        T,
        FlatFee: Iterator<Item = T>,
        FlatSlab: Iterator<Item = T>,
        ReserveV2: Iterator<Item = T>,
    > Iterator for PricingAg<FlatFee, FlatSlab, ReserveV2>
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
impl<A, FlatFee, FlatSlab, ReserveV2> AsRef<A> for PricingAg<FlatFee, FlatSlab, ReserveV2>
where
    A: ?Sized,
    FlatFee: AsRef<A>,
    FlatSlab: AsRef<A>,
    ReserveV2: AsRef<A>,
{
    #[inline]
    fn as_ref(&self) -> &A {
        map_variant_pure!(self, AsRef::as_ref)
    }
}

// Display + Error blanket

impl<FlatFee: Error, FlatSlab: Error, ReserveV2: Error> Display
    for PricingAg<FlatFee, FlatSlab, ReserveV2>
{
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        map_variant_pure!(self, (|p| Display::fmt(&p, f)))
    }
}

impl<FlatFee: Error, FlatSlab: Error, ReserveV2: Error> Error
    for PricingAg<FlatFee, FlatSlab, ReserveV2>
{
}

pub type PricingAgTy = PricingAg<(), (), ()>;

impl PricingAgTy {
    #[inline]
    pub const fn try_from_program_id(program_id: &[u8; 32]) -> Option<Self> {
        Some(match *program_id {
            inf1_pp_flatfee_core::ID => Self::FlatFee(()),
            inf1_pp_flatslab_core::ID => Self::FlatSlab(()),
            inf1_pp_reserve_v2_core::ID => Self::ReserveV2(()),
            _ => return None,
        })
    }
}
