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

// AsRef blanket
impl<A, Lido, Marinade, SanctumSpl, SanctumSplMulti, Spl, Wsol> AsRef<A>
    for SvcAg<Lido, Marinade, SanctumSpl, SanctumSplMulti, Spl, Wsol>
where
    A: ?Sized,
    Lido: AsRef<A>,
    Marinade: AsRef<A>,
    SanctumSpl: AsRef<A>,
    SanctumSplMulti: AsRef<A>,
    Spl: AsRef<A>,
    Wsol: AsRef<A>,
{
    #[inline]
    fn as_ref(&self) -> &A {
        match self {
            Self::Lido(c) => c.as_ref(),
            Self::Marinade(c) => c.as_ref(),
            Self::SanctumSpl(c) => c.as_ref(),
            Self::SanctumSplMulti(c) => c.as_ref(),
            Self::Spl(c) => c.as_ref(),
            Self::Wsol(c) => c.as_ref(),
        }
    }
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

// `owned -> &` const conv
impl<Lido, Marinade, SanctumSpl, SanctumSplMulti, Spl, Wsol>
    SvcAg<Lido, Marinade, SanctumSpl, SanctumSplMulti, Spl, Wsol>
{
    #[inline]
    pub const fn as_ref_const(
        &self,
    ) -> SvcAg<&Lido, &Marinade, &SanctumSpl, &SanctumSplMulti, &Spl, &Wsol> {
        match self {
            SvcAg::Lido(a) => SvcAg::Lido(a),
            SvcAg::Marinade(a) => SvcAg::Marinade(a),
            SvcAg::SanctumSpl(a) => SvcAg::SanctumSpl(a),
            SvcAg::SanctumSplMulti(a) => SvcAg::SanctumSplMulti(a),
            SvcAg::Spl(a) => SvcAg::Spl(a),
            SvcAg::Wsol(a) => SvcAg::Wsol(a),
        }
    }
}

// `& -> owned` const conv for Copy types
impl<
        Lido: Copy,
        Marinade: Copy,
        SanctumSpl: Copy,
        SanctumSplMulti: Copy,
        Spl: Copy,
        Wsol: Copy,
    > SvcAg<&Lido, &Marinade, &SanctumSpl, &SanctumSplMulti, &Spl, &Wsol>
{
    #[inline]
    pub const fn to_owned_copy(
        self,
    ) -> SvcAg<Lido, Marinade, SanctumSpl, SanctumSplMulti, Spl, Wsol> {
        match self {
            SvcAg::Lido(a) => SvcAg::Lido(*a),
            SvcAg::Marinade(a) => SvcAg::Marinade(*a),
            SvcAg::SanctumSpl(a) => SvcAg::SanctumSpl(*a),
            SvcAg::SanctumSplMulti(a) => SvcAg::SanctumSplMulti(*a),
            SvcAg::Spl(a) => SvcAg::Spl(*a),
            SvcAg::Wsol(a) => SvcAg::Wsol(*a),
        }
    }
}

impl<Lido, Marinade, SanctumSpl, SanctumSplMulti, Spl, Wsol>
    SvcAg<Lido, Marinade, SanctumSpl, SanctumSplMulti, Spl, Wsol>
{
    #[inline]
    pub const fn ty(&self) -> SvcAgTy {
        match self {
            Self::Lido(_) => SvcAgTy::Lido(()),
            Self::Marinade(_) => SvcAgTy::Marinade(()),
            Self::SanctumSpl(_) => SvcAgTy::SanctumSpl(()),
            Self::SanctumSplMulti(_) => SvcAgTy::SanctumSplMulti(()),
            Self::Spl(_) => SvcAgTy::Spl(()),
            Self::Wsol(_) => SvcAgTy::Wsol(()),
        }
    }

    #[inline]
    pub const fn svc_program_id(&self) -> &[u8; 32] {
        match self {
            Self::Lido(_) => &inf1_svc_lido_core::ID,
            Self::Marinade(_) => &inf1_svc_marinade_core::ID,
            Self::SanctumSpl(_) => &inf1_svc_spl_core::keys::sanctum_spl::ID,
            Self::SanctumSplMulti(_) => &inf1_svc_spl_core::keys::sanctum_spl_multi::ID,
            Self::Spl(_) => &inf1_svc_spl_core::keys::spl::ID,
            Self::Wsol(_) => &inf1_svc_wsol_core::ID,
        }
    }
}

pub type SvcAgTy = SvcAg<(), (), (), (), (), ()>;

impl SvcAgTy {
    #[inline]
    pub const fn try_from_svc_program_id(program_id: &[u8; 32]) -> Option<Self> {
        Some(match *program_id {
            inf1_svc_lido_core::ID => Self::Lido(()),
            inf1_svc_marinade_core::ID => Self::Marinade(()),
            inf1_svc_spl_core::keys::sanctum_spl::ID => Self::SanctumSpl(()),
            inf1_svc_spl_core::keys::sanctum_spl_multi::ID => Self::SanctumSplMulti(()),
            inf1_svc_spl_core::keys::spl::ID => Self::Spl(()),
            inf1_svc_wsol_core::ID => Self::Wsol(()),
            _ => return None,
        })
    }
}
