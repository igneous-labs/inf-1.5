#![cfg_attr(not(test), no_std)]

use core::{convert::identity, error::Error, fmt::Display};

// Re-exports
pub use inf1_svc_core;
pub use inf1_svc_generic;
pub use inf1_svc_inf_core;
pub use inf1_svc_lido_core;
pub use inf1_svc_marinade_core;
pub use inf1_svc_spl_core;
pub use inf1_svc_wsol_core;

pub mod calc;
pub mod instructions;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SvcAg<Inf, Lido, Marinade, SanctumSpl, SanctumSplMulti, Spl, Wsol> {
    Inf(Inf),
    Lido(Lido),
    Marinade(Marinade),
    SanctumSpl(SanctumSpl),
    SanctumSplMulti(SanctumSplMulti),
    Spl(Spl),
    Wsol(Wsol),
}

/// Example
///
/// ```ignore
/// each_variant_pure!(&self, (|p| Display::fmt(&p, f)))
/// ```
///
/// expands to
///
/// ```ignore
/// match self.0 {
///     SvcAg::Inf(p) => (|p| Display::fmt(&p, f))(p),
///     SvcAg::Lido(p) => (|p| Display::fmt(&p, f))(p),
///     ...
/// }
/// ```
macro_rules! each_variant_pure {
    ($ag:expr, $($e:tt)*) => {{
        use $crate::SvcAg::*;
        match $ag {
            Inf(p) => ($($e)*(p)),
            Lido(p) => ($($e)*(p)),
            Marinade(p) => ($($e)*(p)),
            SanctumSpl(p) => ($($e)*(p)),
            SanctumSplMulti(p) => ($($e)*(p)),
            Spl(p) => ($($e)*(p)),
            Wsol(p) => ($($e)*(p)),
        }
    }};
}
pub(crate) use each_variant_pure;

#[macro_export]
macro_rules! each_variant_method {
    ($ag:expr, $($e:tt)*) => {{
        use $crate::SvcAg::*;

        match $ag {
            Inf(p) => (p.$($e)*),
            Lido(p) => (p.$($e)*),
            Marinade(p) => (p.$($e)*),
            SanctumSpl(p) => (p.$($e)*),
            SanctumSplMulti(p) => (p.$($e)*),
            Spl(p) => (p.$($e)*),
            Wsol(p) => (p.$($e)*),
        }
    }};
}

macro_rules! map_variant_pure {
    ($ag:expr, $($e:tt)*) => {{
        use $crate::SvcAg::*;
        match $ag {
            Inf(p) => Inf($($e)*(p)),
            Lido(p) => Lido($($e)*(p)),
            Marinade(p) => Marinade($($e)*(p)),
            SanctumSpl(p) => SanctumSpl($($e)*(p)),
            SanctumSplMulti(p) => SanctumSplMulti($($e)*(p)),
            Spl(p) => Spl($($e)*(p)),
            Wsol(p) => Wsol($($e)*(p)),
        }
    }};
}
pub(crate) use map_variant_pure;

#[macro_export]
macro_rules! map_variant_method {
    ($ag:expr, $($e:tt)*) => {{
        use $crate::SvcAg::*;

        match $ag {
            Inf(p) => Inf(p.$($e)*),
            Lido(p) => Lido(p.$($e)*),
            Marinade(p) => Marinade(p.$($e)*),
            SanctumSpl(p) => SanctumSpl(p.$($e)*),
            SanctumSplMulti(p) => SanctumSplMulti(p.$($e)*),
            Spl(p) => Spl(p.$($e)*),
            Wsol(p) => Wsol(p.$($e)*),
        }
    }};
}

macro_rules! each_fallible_variant_method {
    ($ag:expr, $($e:tt)*) => {{
        use $crate::SvcAg::*;
        match $ag {
            Inf(p) => match (p.$($e)*) {
                Err(e) => Err(Inf(e)),
                Ok(r) => Ok(r),
            }
            Lido(p) => match (p.$($e)*) {
                Err(e) => Err(Lido(e)),
                Ok(r) => Ok(r),
            }
            Marinade(p) => match (p.$($e)*) {
                Err(e) => Err(Marinade(e)),
                Ok(r) => Ok(r),
            }
            SanctumSpl(p) => match (p.$($e)*) {
                Err(e) => Err(SanctumSpl(e)),
                Ok(r) => Ok(r),
            }
            SanctumSplMulti(p) => match (p.$($e)*) {
                Err(e) => Err(SanctumSplMulti(e)),
                Ok(r) => Ok(r),
            }
            Spl(p) => match (p.$($e)*) {
                Err(e) => Err(Spl(e)),
                Ok(r) => Ok(r),
            }
            Wsol(p) => match (p.$($e)*) {
                Err(e) => Err(Wsol(e)),
                Ok(r) => Ok(r),
            }
        }
    }};
}
pub(crate) use each_fallible_variant_method;

// AsRef blanket
impl<
        A: ?Sized,
        Inf: AsRef<A>,
        Lido: AsRef<A>,
        Marinade: AsRef<A>,
        SanctumSpl: AsRef<A>,
        SanctumSplMulti: AsRef<A>,
        Spl: AsRef<A>,
        Wsol: AsRef<A>,
    > AsRef<A> for SvcAg<Inf, Lido, Marinade, SanctumSpl, SanctumSplMulti, Spl, Wsol>
{
    #[inline]
    fn as_ref(&self) -> &A {
        each_variant_pure!(self, AsRef::as_ref)
    }
}

// Iterator blanket
impl<
        T,
        Inf: Iterator<Item = T>,
        Lido: Iterator<Item = T>,
        Marinade: Iterator<Item = T>,
        SanctumSpl: Iterator<Item = T>,
        SanctumSplMulti: Iterator<Item = T>,
        Spl: Iterator<Item = T>,
        Wsol: Iterator<Item = T>,
    > Iterator for SvcAg<Inf, Lido, Marinade, SanctumSpl, SanctumSplMulti, Spl, Wsol>
{
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        each_variant_pure!(self, Iterator::next)
    }

    #[inline]
    fn fold<B, F>(self, init: B, f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        each_variant_method!(self, fold(init, f))
    }
}

// Display + Error blanket

impl<
        Inf: Error,
        Lido: Error,
        Marinade: Error,
        SanctumSpl: Error,
        SanctumSplMulti: Error,
        Spl: Error,
        Wsol: Error,
    > Display for SvcAg<Inf, Lido, Marinade, SanctumSpl, SanctumSplMulti, Spl, Wsol>
{
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        each_variant_pure!(self, (|e| Display::fmt(e, f)))
    }
}

impl<
        Inf: Error,
        Lido: Error,
        Marinade: Error,
        SanctumSpl: Error,
        SanctumSplMulti: Error,
        Spl: Error,
        Wsol: Error,
    > Error for SvcAg<Inf, Lido, Marinade, SanctumSpl, SanctumSplMulti, Spl, Wsol>
{
}

// `owned -> &` const conv
impl<Inf, Lido, Marinade, SanctumSpl, SanctumSplMulti, Spl, Wsol>
    SvcAg<Inf, Lido, Marinade, SanctumSpl, SanctumSplMulti, Spl, Wsol>
{
    #[inline]
    pub const fn as_ref_const(
        &self,
    ) -> SvcAg<&Inf, &Lido, &Marinade, &SanctumSpl, &SanctumSplMulti, &Spl, &Wsol> {
        map_variant_pure!(self, identity)
    }
}

// `& -> owned` const conv for Copy types
impl<
        Inf: Copy,
        Lido: Copy,
        Marinade: Copy,
        SanctumSpl: Copy,
        SanctumSplMulti: Copy,
        Spl: Copy,
        Wsol: Copy,
    > SvcAg<&Inf, &Lido, &Marinade, &SanctumSpl, &SanctumSplMulti, &Spl, &Wsol>
{
    #[inline]
    pub const fn to_owned_copy(
        self,
    ) -> SvcAg<Inf, Lido, Marinade, SanctumSpl, SanctumSplMulti, Spl, Wsol> {
        // need this
        // - for const fn, closures unallowed
        // - or else rustc cant infer closure types
        const fn deref<T: Copy>(x: &T) -> T {
            *x
        }
        map_variant_pure!(self, deref)
    }
}

impl<Inf, Lido, Marinade, SanctumSpl, SanctumSplMulti, Spl, Wsol>
    SvcAg<Inf, Lido, Marinade, SanctumSpl, SanctumSplMulti, Spl, Wsol>
{
    #[inline]
    pub const fn ty(&self) -> SvcAgTy {
        // need this for const fn, closures unallowed
        const fn empty<T>(_x: &T) {}
        map_variant_pure!(self, empty)
    }

    #[inline]
    pub const fn svc_program_id(&self) -> &[u8; 32] {
        match self {
            Self::Inf(_) => &inf1_ctl_core::ID,
            Self::Lido(_) => &inf1_svc_lido_core::ID,
            Self::Marinade(_) => &inf1_svc_marinade_core::ID,
            Self::SanctumSpl(_) => &inf1_svc_spl_core::keys::sanctum_spl::ID,
            Self::SanctumSplMulti(_) => &inf1_svc_spl_core::keys::sanctum_spl_multi::ID,
            Self::Spl(_) => &inf1_svc_spl_core::keys::spl::ID,
            Self::Wsol(_) => &inf1_svc_wsol_core::ID,
        }
    }
}

pub type SvcAgTy = SvcAg<(), (), (), (), (), (), ()>;

impl SvcAgTy {
    #[inline]
    pub const fn try_from_svc_program_id(program_id: &[u8; 32]) -> Option<Self> {
        Some(match *program_id {
            inf1_ctl_core::ID => Self::Inf(()),
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
