use core::{error::Error, fmt::Display, ops::Deref};

use crate::typedefs::uq0f63::UQ0F63;

pub const MIN_RPS_RAW: u64 = 9_223_372;

/// Approx one pico (1 / 1_000_000_000_000)
pub const MIN_RPS: UQ0F63 = match UQ0F63::new(MIN_RPS_RAW) {
    Err(_) => unreachable!(),
    Ok(x) => x,
};

/// Proportion of withheld_lamports to release per slot
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Rps(UQ0F63);

impl Rps {
    pub const MIN: Self = Self(MIN_RPS);

    /// Define
    /// - k as rps in terms of ate (0.0 - 1.0)
    /// - τ as the period of time where after which, we want 0.9999 of any yield collected to be distributed.
    ///
    /// ```
    /// 1 - 0.9999 ≥ (1-k)^τ
    /// 0.0001 ≥ (1-k)^τ
    /// k ≥ 1 - (0.0001)^(1/τ)
    /// ```
    ///
    /// Set τ = 2160000 (5 epochs). k = 4.264037377521568e-06
    /// ≈ 39328803111936 / (1 << 63)
    pub const DEFAULT: Self = match UQ0F63::new(39328803111936) {
        // use checked fns here to ensure we dont have an invalid const
        Err(_) => unreachable!(),
        Ok(x) => match Self::new(x) {
            Err(_) => unreachable!(),
            Ok(x) => x,
        },
    };

    #[inline]
    pub const fn new(raw: UQ0F63) -> Result<Self, RpsTooSmallErr> {
        // have to cmp raw values to use primitive const < operator
        if *raw.as_raw() < *MIN_RPS.as_raw() {
            Err(RpsTooSmallErr { actual: raw })
        } else {
            Ok(Self(raw))
        }
    }

    #[inline]
    pub const fn as_inner(&self) -> &UQ0F63 {
        &self.0
    }
}

impl Default for Rps {
    #[inline]
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl Deref for Rps {
    type Target = UQ0F63;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_inner()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RpsTooSmallErr {
    pub actual: UQ0F63,
}

impl Display for RpsTooSmallErr {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let Self { actual } = self;
        f.write_fmt(format_args!("{actual} < {MIN_RPS} (min)"))
    }
}

impl Error for RpsTooSmallErr {}

#[cfg(test)]
pub mod test_utils {
    use proptest::prelude::*;

    use super::*;

    pub fn any_rps_strat() -> impl Strategy<Value = Rps> {
        (MIN_RPS_RAW..=*UQ0F63::ONE.as_raw())
            .prop_map(UQ0F63::new)
            .prop_map(Result::unwrap)
            .prop_map(Rps::new)
            .prop_map(Result::unwrap)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rps_new_sc() {
        const FAIL: UQ0F63 = unsafe { UQ0F63::new_unchecked(MIN_RPS_RAW - 1) };
        const SUCC: UQ0F63 = unsafe { UQ0F63::new_unchecked(MIN_RPS_RAW) };

        assert_eq!(Rps::new(FAIL), Err(RpsTooSmallErr { actual: FAIL }));
        assert_eq!(Rps::new(SUCC), Ok(Rps(SUCC)));
    }
}
