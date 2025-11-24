//! Binary fixed-point Q numbers
//! (https://en.wikipedia.org/wiki/Q_(number_format))
//!
//! ## Why handroll our own?
//! - `fixed` crate has dependencies that we dont need
//! - we only need multiplication and exponentiation of unsigned ratios <= 1.0
//!
//! ### TODO
//! Consider generalizing and separating this out into its own crate?

use core::{fmt::Display, ops::Mul};

use sanctum_u64_ratio::Ratio;

/// 64-bit fraction only fixed-point number to represent a value between 0.0 and 1.0
/// (denominator = u64::MAX)
#[repr(transparent)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UQ0_64(pub u64);

#[inline]
pub const fn uq0_64_mul(UQ0_64(a): UQ0_64, UQ0_64(b): UQ0_64) -> UQ0_64 {
    const ROUNDING_BIAS: u128 = 1 << 63;

    // where d = u64::MAX
    // (n1/d) * (n2/d) = n1*n2/d^2
    // as-safety: u128 bitwidth > u64 bitwidth
    // unchecked arith safety: u64*u64 never overflows u128
    let res = (a as u128) * (b as u128);
    // round off 64th bit
    // unchecked arith safety: res <= u64::MAX * u64::MAX + ROUNDING_BIAS < u128::MAX
    let rounded = res + ROUNDING_BIAS;
    // convert back to UQ0_64 by making denom = d
    // n1*n2/d^2 = (n1*n2/d) / d
    // so we need to divide res by d,
    // and division by u64::MAX is just >> 64
    // as-safety: truncating conversion is what we want
    // to achieve floor mul
    UQ0_64((rounded >> 64) as u64)
}

#[inline]
pub const fn uq0_64_into_ratio(a: UQ0_64) -> Ratio<u64, u64> {
    Ratio {
        n: a.0,
        d: u64::MAX,
    }
}

#[inline]
pub const fn uq0_64_pow(mut base: UQ0_64, mut exp: u64) -> UQ0_64 {
    // sq & mul
    let mut res = UQ0_64::ONE;
    while exp > 0 {
        if exp % 2 == 1 {
            res = uq0_64_mul(res, base);
        }
        base = uq0_64_mul(base, base);
        exp /= 2;
    }
    res
}

impl UQ0_64 {
    pub const ZERO: Self = Self(0);
    pub const ONE: Self = Self(u64::MAX);

    /// Rounding is to closest bit
    #[inline]
    pub const fn const_mul(a: Self, b: Self) -> Self {
        uq0_64_mul(a, b)
    }

    /// Returns `1.0 - self`
    #[inline]
    pub const fn one_minus(self) -> Self {
        // unchecked-arith safety: self.0 <= u64::MAX
        Self(u64::MAX - self.0)
    }

    #[inline]
    pub const fn pow(self, exp: u64) -> Self {
        uq0_64_pow(self, exp)
    }

    #[inline]
    pub const fn into_ratio(self) -> Ratio<u64, u64> {
        uq0_64_into_ratio(self)
    }
}

impl Mul for UQ0_64 {
    type Output = Self;

    /// Rounding floors
    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        Self::const_mul(self, rhs)
    }
}

impl From<UQ0_64> for Ratio<u64, u64> {
    #[inline]
    fn from(v: UQ0_64) -> Self {
        v.into_ratio()
    }
}

impl Display for UQ0_64 {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.into_ratio().fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use core::cmp::min;

    use expect_test::expect;
    use proptest::prelude::*;
    use sanctum_u64_ratio::Ratio;

    use super::*;

    const D: f64 = u64::MAX as f64;

    /// max error bounds for multiplication
    /// - UQ0_64. 1-bit, so 2^-64
    /// - f64 for range 0.0-1.0, around 2^-54 (around 2^10 larger than UQ0_64 because fewer bits dedicated to fraction)
    const MAX_MUL_DIFF_F64_VS_US: u64 = 4096;

    const EPSILON_RATIO_DIFF: Ratio<u64, u64> = Ratio {
        n: 1,
        d: 1_000_000_000_000,
    };

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
        fn mul_pt(
            [a, b] in [any::<u64>(); 2].map(|s| s.prop_map(UQ0_64))
        ) {
            let us = a * b;

            // a*b <= a and <= b since both are <= 1.0
            prop_assert!(us <= a);
            prop_assert!(us <= b);

            let approx_f64 = [a, b].map(f64_approx).into_iter().reduce(core::ops::Mul::mul).unwrap();
            let approx_uq0_64 = uq0_64_approx(approx_f64);

            // small error from f64 result
            let diff_u64 = us.0.abs_diff(approx_uq0_64.0);
            prop_assert!(
                diff_u64 <= MAX_MUL_DIFF_F64_VS_US,
                "{}, {}",
                us.0,
                approx_uq0_64.0
            );

            // diff should not exceed epsilon proportion of value
            let diff_r = Ratio {
                n: diff_u64,
                d: min(us.0, approx_uq0_64.0),
            };
            prop_assert!(diff_r < EPSILON_RATIO_DIFF, "diff_r: {diff_r}");
        }
    }

    proptest! {
        #[test]
        fn exp_pt(base in any::<u64>().prop_map(UQ0_64), exp: u64) {
            let us = base.pow(exp);

            // (base)^+ve should be <= base since base <= 1.0
            prop_assert!(us <= base);

            let approx_f64 = f64_approx(us).powf(exp as f64);
            let approx_uq0_64 = uq0_64_approx(approx_f64);

            // small error from f64 result
            let diff_u64 = us.0.abs_diff(approx_uq0_64.0);
            // same err bound as mul_pt since a.pow(2) = a * a
            prop_assert!(
                diff_u64 <= MAX_MUL_DIFF_F64_VS_US,
                "{}, {}",
                us.0,
                approx_uq0_64.0
            );

            // diff should not exceed epsilon proportion of value
            let diff_r = Ratio {
                n: diff_u64,
                d: min(us.0, approx_uq0_64.0),
            };
            prop_assert!(diff_r < EPSILON_RATIO_DIFF, "diff_r: {diff_r}");

            // exponent of anything < 1.0 eventually reaches 0
            if base != UQ0_64::ONE {
                prop_assert_eq!(base.pow(u64::MAX), UQ0_64::ZERO);
            }

            // compare against naive multiplication implementation
            const LIM: u64 = u16::MAX as u64;
            let naive_mul_res = match exp {
                0 => UQ0_64::ONE,
                1..=LIM => (0..exp).fold(base, |res, _| res * base),
                _will_take_too_long_to_run => return Ok(())
            };
            prop_assert_eq!(naive_mul_res, us);
        }
    }

    #[test]
    fn into_ratio_sc() {
        expect!["9223372036854775807/18446744073709551615"]
            .assert_eq(&UQ0_64(u64::MAX / 2).into_ratio().to_string());
    }
}
