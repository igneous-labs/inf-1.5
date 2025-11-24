use core::{error::Error, fmt::Display, ops::Deref};

use crate::typedefs::uq0_63::UQ0_63;

const MIN_RPS_RAW: u64 = 9_223_372;

/// Approx one pico (1 / 1_000_000_000_000)
pub const MIN_RPS: UQ0_63 = unsafe { UQ0_63::new_unchecked(MIN_RPS_RAW) };

/// Proportion of withheld_lamports to release per slot
#[repr(transparent)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Rps(UQ0_63);

impl Rps {
    pub const MIN: Self = Self(MIN_RPS);

    #[inline]
    pub const fn new(raw: UQ0_63) -> Result<Self, RpsTooSmallErr> {
        // have to cmp raw values to use primitive const < operator
        if *raw.as_raw() < *MIN_RPS.as_raw() {
            Err(RpsTooSmallErr { actual: raw })
        } else {
            Ok(Self(raw))
        }
    }

    #[inline]
    pub const fn as_inner(&self) -> &UQ0_63 {
        &self.0
    }
}

impl Deref for Rps {
    type Target = UQ0_63;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_inner()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RpsTooSmallErr {
    pub actual: UQ0_63,
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
        (MIN_RPS_RAW..=*UQ0_63::ONE.as_raw())
            .prop_map(UQ0_63::new)
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
        const FAIL: UQ0_63 = unsafe { UQ0_63::new_unchecked(MIN_RPS_RAW - 1) };
        const SUCC: UQ0_63 = unsafe { UQ0_63::new_unchecked(MIN_RPS_RAW) };

        assert_eq!(Rps::new(FAIL), Err(RpsTooSmallErr { actual: FAIL }));
        assert_eq!(Rps::new(SUCC), Ok(Rps(SUCC)));
    }
}
