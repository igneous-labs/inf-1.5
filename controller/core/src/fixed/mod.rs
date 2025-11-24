//! Binary fixed-point Q numbers
//! (https://en.wikipedia.org/wiki/Q_(number_format))
//!
//! ## Why handroll our own?
//! - `fixed` crate has dependencies that we dont need
//! - we only need multiplication and exponentiation of unsigned ratios <= 1.0
//!
//! ### TODO
//! Consider generalizing and separating this out into its own crate?

use core::ops::Mul;

use sanctum_u64_ratio::Ratio;

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UQ0_64(pub u64);

#[inline]
pub const fn uq0_64_floor_mul(UQ0_64(a): UQ0_64, UQ0_64(b): UQ0_64) -> UQ0_64 {
    // where d = u64::MAX
    // (n1/d) * (n2/d) = n1*n2/d^2
    // as-safety: u128 bitwidth > u64 bitwidth
    // unchecked arith safety: u64*u64 never overflows u128
    let res = (a as u128) * (b as u128);
    // convert back to UQ0_64 by making denom = d
    // n1*n2/d^2 = (n1*n2/d) / d
    // so we need to divide res by d,
    // and division by u64::MAX is just >> 64
    // as-safety: truncating conversion is what we want
    // to achieve floor mul
    UQ0_64((res >> 64) as u64)
}

impl UQ0_64 {
    #[inline]
    pub const fn floor_mul(a: Self, b: Self) -> Self {
        uq0_64_floor_mul(a, b)
    }

    #[inline]
    pub const fn into_ratio(self) -> Ratio<u64, u64> {
        Ratio {
            n: self.0,
            d: u64::MAX,
        }
    }
}

impl Mul for UQ0_64 {
    type Output = Self;

    /// Rounding floors
    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        Self::floor_mul(self, rhs)
    }
}

impl From<UQ0_64> for Ratio<u64, u64> {
    #[inline]
    fn from(v: UQ0_64) -> Self {
        v.into_ratio()
    }
}

#[cfg(test)]
mod tests {
    use core::cmp::min;

    use proptest::prelude::*;
    use sanctum_u64_ratio::Ratio;

    use super::*;

    const D: f64 = u64::MAX as f64;

    const fn f64_approx(UQ0_64(a): UQ0_64) -> f64 {
        (a as f64) / D
    }

    fn uq0_64_approx(a: f64) -> UQ0_64 {
        if a > 1.0 {
            panic!("a={a} > 1.0");
        }
        UQ0_64((a * D).floor() as u64)
    }

    proptest! {
        #[test]
        fn mul_pt(a: u64, b: u64) {
            let [a, b] = [a, b].map(UQ0_64);
            let us = a * b;

            let approx_f64 = [a, b].map(f64_approx).into_iter().reduce(core::ops::Mul::mul).unwrap();
            let approx_uq0_64 = uq0_64_approx(approx_f64);

            // max error bounds for multiplication
            // - UQ0_64. 1-bit, so 2^-64
            // - f64 for range 0.0-1.0, around 2^-54 (around 2^10 larger than UQ0_64 because fewer bits dedicated to fraction)
            let diff_u64 = us.0.abs_diff(approx_uq0_64.0);
            prop_assert!(
                diff_u64 <= 4096,
                "{}, {}",
                us.0,
                approx_uq0_64.0
            );
            // diff should not exceed 1 / 1_000_000_000 of value
            let diff_r = Ratio {
                n: diff_u64,
                d: min(us.0, approx_uq0_64.0),
            };
            prop_assert!(diff_r < Ratio { n: 1, d: 1_000_000_000 }, "diff_r: {diff_r}");
        }
    }
}
