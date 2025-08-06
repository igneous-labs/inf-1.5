#![cfg_attr(not(test), no_std)]

use core::{error::Error, fmt::Display};

pub trait Account {
    fn data(&self) -> &[u8];
}

// cant generalize over for Deref<T> due to lifetime of &[u8]
/// Blanket for refs
impl<T: Account> Account for &T {
    #[inline]
    fn data(&self) -> &[u8] {
        (*self).data()
    }
}

pub trait UpdateMap {
    type Account<'a>: Account
    where
        Self: 'a;

    fn get_account(&self, pk: &[u8; 32]) -> Option<Self::Account<'_>>;

    #[inline]
    fn get_account_checked<E>(&self, pk: &[u8; 32]) -> Result<Self::Account<'_>, UpdateErr<E>> {
        self.get_account(pk)
            .ok_or(UpdateErr::AccMissing { pk: *pk })
    }
}

/// Blanket for refs
impl<T: UpdateMap> UpdateMap for &T {
    type Account<'a>
        = T::Account<'a>
    where
        Self: 'a;

    #[inline]
    fn get_account(&self, pk: &[u8; 32]) -> Option<Self::Account<'_>> {
        (*self).get_account(pk)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UpdateErr<E> {
    /// Account missing from `UpdateMap`
    AccMissing {
        pk: [u8; 32],
    },
    Inner(E),
}

impl<E> UpdateErr<E> {
    #[inline]
    pub fn map_inner<T>(self, f: impl FnOnce(E) -> T) -> UpdateErr<T> {
        match self {
            Self::AccMissing { pk } => UpdateErr::AccMissing { pk },
            Self::Inner(e) => UpdateErr::Inner(f(e)),
        }
    }
}

impl<E: Display> Display for UpdateErr<E> {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::AccMissing { .. } => f.write_str("AccMissing"),
            Self::Inner(e) => write!(f, "Inner::{e}"),
        }
    }
}

impl<E: core::fmt::Debug + Display> Error for UpdateErr<E> {}
