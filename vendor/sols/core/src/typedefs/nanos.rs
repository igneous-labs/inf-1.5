use core::{error::Error, fmt::Display, ops::Deref};

use sanctum_u64_ratio::Ratio;

/// A decimal between 0.0 and 1.0, in terms of 1-billionths
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Nanos(u32);

impl Nanos {
    pub const DENOM: u32 = 1_000_000_000;

    #[inline]
    pub const fn new(n: u32) -> Result<Self, NanosOutOfRangeErr> {
        if n > Self::DENOM {
            Err(NanosOutOfRangeErr { actual: n })
        } else {
            Ok(Self(n))
        }
    }

    #[inline]
    pub const fn get(&self) -> u32 {
        self.0
    }

    #[inline]
    pub const fn into_ratio(self) -> Ratio<u32, u32> {
        Ratio {
            n: self.0,
            d: Self::DENOM,
        }
    }

    #[inline]
    pub const fn into_ratio_u64(self) -> Ratio<u64, u64> {
        let Ratio { n, d } = self.into_ratio();
        // as-safety: smaller bitwidth as larger bitwidth unsigned
        Ratio {
            n: n as u64,
            d: d as u64,
        }
    }

    /// # Safety
    /// - `r` must be in range (<= [`Self::DENOM`])
    #[inline]
    pub const unsafe fn of_ref_unchecked(r: &u32) -> &Self {
        let this = core::ptr::from_ref(r).cast();
        // safety: repr(transparent)
        unsafe { &*this }
    }

    #[inline]
    pub const fn of_ref(r: &u32) -> Result<&Self, NanosOutOfRangeErr> {
        if *r > Self::DENOM {
            Err(NanosOutOfRangeErr { actual: *r })
        } else {
            Ok(unsafe { Self::of_ref_unchecked(r) })
        }
    }

    /// # Safety
    /// - `r` must be in range (<= [`Self::DENOM`])
    #[inline]
    pub const unsafe fn of_ref_mut_unchecked(r: &mut u32) -> &mut Self {
        let this = core::ptr::from_mut(r).cast();
        // safety: repr(transparent)
        unsafe { &mut *this }
    }

    #[inline]
    pub const fn of_ref_mut(r: &mut u32) -> Result<&mut Self, NanosOutOfRangeErr> {
        if *r > Self::DENOM {
            Err(NanosOutOfRangeErr { actual: *r })
        } else {
            Ok(unsafe { Self::of_ref_mut_unchecked(r) })
        }
    }
}

impl Deref for Nanos {
    type Target = u32;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NanosOutOfRangeErr {
    pub actual: u32,
}

impl Display for NanosOutOfRangeErr {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!(
            "nanos {} > {} (1.0)",
            self.actual,
            Nanos::DENOM
        ))
    }
}

impl Error for NanosOutOfRangeErr {}

#[cfg(kani)]
pub mod spec {
    use kani::{any, assume};

    use super::*;

    pub fn any_nanos() -> Nanos {
        let n = any();
        assume(n <= Nanos::DENOM);
        Nanos(n)
    }
}
