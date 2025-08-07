#![cfg_attr(not(test), no_std)]

use core::{error::Error, fmt::Display};

// Re-exports
pub use inf1_svc_core;
pub use inf1_svc_generic;
pub use inf1_svc_lido_core;
pub use inf1_svc_marinade_core;
pub use inf1_svc_spl_core;
pub use inf1_svc_wsol_core;

pub mod calc;
pub mod instructions;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SvcAg<Lido, Marinade, SanctumSpl, SanctumSplMulti, Spl, Wsol> {
    Lido(Lido),
    Marinade(Marinade),
    SanctumSpl(SanctumSpl),
    SanctumSplMulti(SanctumSplMulti),
    Spl(Spl),
    Wsol(Wsol),
}

// Iterator blanket
impl<
        T,
        Lido: Iterator<Item = T>,
        Marinade: Iterator<Item = T>,
        SanctumSpl: Iterator<Item = T>,
        SanctumSplMulti: Iterator<Item = T>,
        Spl: Iterator<Item = T>,
        Wsol: Iterator<Item = T>,
    > Iterator for SvcAg<Lido, Marinade, SanctumSpl, SanctumSplMulti, Spl, Wsol>
{
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Lido(c) => c.next(),
            Self::Marinade(c) => c.next(),
            Self::SanctumSpl(c) => c.next(),
            Self::SanctumSplMulti(c) => c.next(),
            Self::Spl(c) => c.next(),
            Self::Wsol(c) => c.next(),
        }
    }

    #[inline]
    fn fold<B, F>(self, init: B, f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        match self {
            Self::Lido(c) => c.fold(init, f),
            Self::Marinade(c) => c.fold(init, f),
            Self::SanctumSpl(c) => c.fold(init, f),
            Self::SanctumSplMulti(c) => c.fold(init, f),
            Self::Spl(c) => c.fold(init, f),
            Self::Wsol(c) => c.fold(init, f),
        }
    }
}

// Display + Error blanket

impl<
        Lido: Error,
        Marinade: Error,
        SanctumSpl: Error,
        SanctumSplMulti: Error,
        Spl: Error,
        Wsol: Error,
    > Display for SvcAg<Lido, Marinade, SanctumSpl, SanctumSplMulti, Spl, Wsol>
{
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Lido(e) => Display::fmt(&e, f),
            Self::Marinade(e) => Display::fmt(&e, f),
            Self::SanctumSpl(e) => Display::fmt(&e, f),
            Self::SanctumSplMulti(e) => Display::fmt(&e, f),
            Self::Spl(e) => Display::fmt(&e, f),
            Self::Wsol(e) => Display::fmt(&e, f),
        }
    }
}

impl<
        Lido: Error,
        Marinade: Error,
        SanctumSpl: Error,
        SanctumSplMulti: Error,
        Spl: Error,
        Wsol: Error,
    > Error for SvcAg<Lido, Marinade, SanctumSpl, SanctumSplMulti, Spl, Wsol>
{
}

impl<Lido, Marinade, SanctumSpl, SanctumSplMulti, Spl, Wsol>
    SvcAg<Lido, Marinade, SanctumSpl, SanctumSplMulti, Spl, Wsol>
{
    #[inline]
    pub const fn ty(&self) -> SvcAgTy {
        match self {
            Self::Lido(_) => SvcAgTy::Lido,
            Self::Marinade(_) => SvcAgTy::Marinade,
            Self::SanctumSpl(_) => SvcAgTy::SanctumSpl,
            Self::SanctumSplMulti(_) => SvcAgTy::SanctumSplMulti,
            Self::Spl(_) => SvcAgTy::Spl,
            Self::Wsol(_) => SvcAgTy::Wsol,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SvcAgTy {
    Lido,
    Marinade,
    SanctumSpl,
    SanctumSplMulti,
    Spl,
    Wsol,
}

impl SvcAgTy {
    #[inline]
    pub const fn program_id(&self) -> &[u8; 32] {
        match self {
            Self::Lido => &inf1_svc_lido_core::ID,
            Self::Marinade => &inf1_svc_marinade_core::ID,
            Self::SanctumSpl => &inf1_svc_spl_core::keys::sanctum_spl::ID,
            Self::SanctumSplMulti => &inf1_svc_spl_core::keys::sanctum_spl_multi::ID,
            Self::Spl => &inf1_svc_spl_core::keys::spl::ID,
            Self::Wsol => &inf1_svc_wsol_core::ID,
        }
    }

    #[inline]
    pub const fn try_from_program_id(program_id: &[u8; 32]) -> Option<Self> {
        Some(match *program_id {
            inf1_svc_lido_core::ID => Self::Lido,
            inf1_svc_marinade_core::ID => Self::Marinade,
            inf1_svc_spl_core::keys::sanctum_spl::ID => Self::SanctumSpl,
            inf1_svc_spl_core::keys::sanctum_spl_multi::ID => Self::SanctumSplMulti,
            inf1_svc_spl_core::keys::spl::ID => Self::Spl,
            inf1_svc_wsol_core::ID => Self::Wsol,
            _ => return None,
        })
    }
}
