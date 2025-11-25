use core::{error::Error, fmt::Display, ops::Deref};

use sanctum_fee_ratio::Fee;
use sanctum_u64_ratio::{Ceil, Ratio};

pub const NANOS_DENOM: u32 = 1_000_000_000;

/// 100%
pub const MAX_FEE_NANOS: u32 = NANOS_DENOM;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(transparent)]
pub struct FeeNanos(u32);

impl FeeNanos {
    /// 0%
    pub const ZERO: Self = Self(0);

    /// 100%
    pub const MAX: Self = Self(MAX_FEE_NANOS);

    #[inline]
    pub const fn new(n: u32) -> Result<Self, FeeNanosTooLargeErr> {
        if n > MAX_FEE_NANOS {
            Err(FeeNanosTooLargeErr { actual: n })
        } else {
            Ok(Self(n))
        }
    }

    #[inline]
    pub const fn get(&self) -> u32 {
        self.0
    }

    #[inline]
    pub const fn into_fee(self) -> F {
        // safety: n <= d checked at construction (::new())
        unsafe {
            F::new_unchecked(Ratio {
                n: self.0,
                d: NANOS_DENOM,
            })
        }
    }
}

type F = Fee<Ceil<Ratio<u32, u32>>>;

impl Deref for FeeNanos {
    type Target = u32;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FeeNanosTooLargeErr {
    pub actual: u32,
}

impl Display for FeeNanosTooLargeErr {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let Self { actual } = self;
        f.write_fmt(format_args!("fee nanos {actual} > {MAX_FEE_NANOS} (max)"))
    }
}

impl Error for FeeNanosTooLargeErr {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_range_sc() {
        const FAIL: u32 = NANOS_DENOM + 1;
        const SUCC: u32 = NANOS_DENOM;

        assert_eq!(
            FeeNanos::new(FAIL),
            Err(FeeNanosTooLargeErr { actual: FAIL })
        );
        assert!(FeeNanos::new(SUCC).is_ok());
    }
}
