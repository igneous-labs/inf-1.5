use core::{error::Error, fmt::Display, ops::Deref};

use crate::typedefs::uq0_64::UQ0_64;

const MIN_RPS_RAW: u64 = 18_446_744;

/// Approx one pico (1 / 1_000_000_000_000)
pub const MIN_RPS: UQ0_64 = UQ0_64(MIN_RPS_RAW);

/// Proportion of withheld_lamports to release per slot
#[repr(transparent)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Rps(UQ0_64);

impl Rps {
    pub const MIN: Self = Self(MIN_RPS);

    #[inline]
    pub const fn new(raw: UQ0_64) -> Result<Self, RpsTooSmallErr> {
        // have to use .0 to use primitive const < operator
        if raw.0 < MIN_RPS.0 {
            Err(RpsTooSmallErr { actual: raw })
        } else {
            Ok(Self(raw))
        }
    }

    #[inline]
    pub const fn as_inner(&self) -> &UQ0_64 {
        &self.0
    }
}

impl Deref for Rps {
    type Target = UQ0_64;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_inner()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RpsTooSmallErr {
    pub actual: UQ0_64,
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
        (MIN_RPS_RAW..=u64::MAX)
            .prop_map(UQ0_64)
            .prop_map(Rps::new)
            .prop_map(Result::unwrap)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rps_new_sc() {
        const FAIL: UQ0_64 = UQ0_64(MIN_RPS_RAW - 1);
        const SUCC: UQ0_64 = UQ0_64(MIN_RPS_RAW);

        assert_eq!(Rps::new(FAIL), Err(RpsTooSmallErr { actual: FAIL }));
        assert_eq!(Rps::new(SUCC), Ok(Rps(SUCC)));
    }
}
