//! Binary fixed-point Q numbers
//! (https://en.wikipedia.org/wiki/Q_(number_format))
//!
//! ## Why handroll our own?
//! - `fixed` crate has dependencies that we dont need
//! - we only need multiplication and exponentiation of unsigned ratios <= 1.0
//!
//! ### TODO
//! Consider generalizing and separating this out into its own crate?

use core::{error::Error, fmt::Display, ops::Mul};

use sanctum_u64_ratio::Ratio;

/// 63-bit fraction only fixed-point number to represent a value between 0.0 and 1.0
/// (denominator = 2^63)
#[repr(transparent)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UQ0_63(u64);

impl UQ0_63 {
    #[inline]
    pub const fn new(n: u64) -> Result<Self, UQ0_63TooLargeErr> {
        if n > D {
            Err(UQ0_63TooLargeErr { actual: n })
        } else {
            Ok(Self(n))
        }
    }

    /// # Safety
    /// - n must be in range (<= 1 << 63)
    #[inline]
    pub const unsafe fn new_unchecked(n: u64) -> Self {
        Self(n)
    }

    #[inline]
    pub const fn as_raw(&self) -> &u64 {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UQ0_63TooLargeErr {
    pub actual: u64,
}

impl Display for UQ0_63TooLargeErr {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let Self { actual } = self;
        f.write_fmt(format_args!("{actual} > {D} (max)"))
    }
}

impl Error for UQ0_63TooLargeErr {}

const Q: u8 = 63;
const Q_SUB_1: u8 = Q - 1;
const D: u64 = 1 << Q;

#[inline]
pub const fn uq0_63_mul(UQ0_63(a): UQ0_63, UQ0_63(b): UQ0_63) -> UQ0_63 {
    // == 0.5
    const ROUNDING_BIAS: u128 = 1 << Q_SUB_1;

    // where d = u64::MAX
    // (n1/d) * (n2/d) = n1*n2/d^2
    //
    // as-safety: u128 bitwidth > u64 bitwidth
    // unchecked arith safety: u64*u64 never overflows u128
    let res = (a as u128) * (b as u128);
    // round off 64th bit
    //
    // unchecked arith safety: res <= D * D + ROUNDING_BIAS < u128::MAX
    let res = res + ROUNDING_BIAS;
    // convert back to UQ0_63 by making denom = d
    // n1*n2/d^2 = (n1*n2/d) / d
    // so we need to divide res by d,
    // and division by 2^Q is just >> Q
    //
    // as-safety: truncating conversion is what we want
    // to achieve floor mul
    UQ0_63((res >> Q) as u64)
}

#[inline]
pub const fn uq0_63_into_ratio(a: UQ0_63) -> Ratio<u64, u64> {
    Ratio { n: a.0, d: D }
}

#[inline]
pub const fn uq0_63_pow(mut base: UQ0_63, mut exp: u64) -> UQ0_63 {
    // sq & mul
    let mut res = UQ0_63::ONE;
    while exp > 0 {
        if exp % 2 == 1 {
            res = uq0_63_mul(res, base);
        }
        base = uq0_63_mul(base, base);
        exp /= 2;
    }
    res
}

impl UQ0_63 {
    pub const ZERO: Self = Self(0);
    pub const ONE: Self = Self(D);

    /// Rounding is to closest bit
    #[inline]
    pub const fn const_mul(a: Self, b: Self) -> Self {
        uq0_63_mul(a, b)
    }

    /// Returns `1.0 - self`
    #[inline]
    pub const fn one_minus(self) -> Self {
        // unchecked-arith safety: self.0 <= D
        Self(D - self.0)
    }

    #[inline]
    pub const fn pow(self, exp: u64) -> Self {
        uq0_63_pow(self, exp)
    }

    #[inline]
    pub const fn into_ratio(self) -> Ratio<u64, u64> {
        uq0_63_into_ratio(self)
    }
}

impl Mul for UQ0_63 {
    type Output = Self;

    /// Rounding floors
    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        Self::const_mul(self, rhs)
    }
}

impl From<UQ0_63> for Ratio<u64, u64> {
    #[inline]
    fn from(v: UQ0_63) -> Self {
        v.into_ratio()
    }
}

impl Display for UQ0_63 {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.into_ratio().fmt(f)
    }
}

#[cfg(test)]
pub mod test_utils {
    use proptest::prelude::Strategy;

    use crate::typedefs::uq0_63::UQ0_63;

    use super::*;

    pub fn any_uq0_63_strat() -> impl Strategy<Value = UQ0_63> {
        (0..=D).prop_map(UQ0_63::new).prop_map(Result::unwrap)
    }
}

#[cfg(test)]
mod tests {
    use core::cmp::min;

    use proptest::prelude::*;
    use sanctum_u64_ratio::Ratio;

    use crate::typedefs::uq0_63::test_utils::any_uq0_63_strat;

    use super::*;

    const D_F64: f64 = D as f64;

    /// max error bounds for multiplication
    /// - UQ0_63. 1-bit, so 2^-63
    /// - f64 for range 0.0-1.0, around 2^-54 (around 2^9 larger than UQ0_63 because fewer bits dedicated to fraction)
    const MAX_MUL_DIFF_F64_VS_US: u64 = 2048;

    const EPSILON_RATIO_DIFF: Ratio<u64, u64> = Ratio {
        n: 1,
        d: 10_000_000,
    };

    const fn f64_approx(UQ0_63(a): UQ0_63) -> f64 {
        (a as f64) / D_F64
    }

    fn uq0_63_approx(a: f64) -> UQ0_63 {
        if a > 1.0 {
            panic!("a={a} > 1.0");
        }
        UQ0_63((a * D_F64).floor() as u64)
    }

    proptest! {
        #[test]
        fn mul_pt(
            [a, b] in core::array::from_fn(|_| any_uq0_63_strat())
        ) {
            let us = a * b;

            // a*b <= a and <= b since both are <= 1.0
            prop_assert!(us <= a, "{us} {a}");
            prop_assert!(us <= b, "{us} {b}");

            let approx_f64 = [a, b].map(f64_approx).into_iter().reduce(core::ops::Mul::mul).unwrap();
            let approx_uq0_63 = uq0_63_approx(approx_f64);

            // small error from f64 result
            let diff_u64 = us.0.abs_diff(approx_uq0_63.0);
            prop_assert!(
                diff_u64 <= MAX_MUL_DIFF_F64_VS_US,
                "{}, {}",
                us.0,
                approx_uq0_63.0
            );

            // diff should not exceed epsilon proportion of value
            let diff_r = Ratio {
                n: diff_u64,
                d: min(us.0, approx_uq0_63.0),
            };
            prop_assert!(
                diff_r < EPSILON_RATIO_DIFF,
                "diff_r: {diff_r}. us: {us}. f64: {approx_uq0_63}"
            );
        }
    }

    proptest! {
        #[test]
        fn exp_pt(
            base in any_uq0_63_strat(),
            // use smaller range to include boundary cases more often
            // larger exps are less interesting since its likely they just go to 0
            exp in 0..=u16::MAX as u64
        ) {
            let us = base.pow(exp);

            if exp == 0 {
                // x^0 == 1
                prop_assert_eq!(us, UQ0_63::ONE);
            } else if base == UQ0_63::ZERO || base == UQ0_63::ONE || exp == 1 {
                // 0^+ve = 0, 1^+ve = 1, x^1 = x
                prop_assert_eq!(us, base);
            } else {
                // x^+ve should be < x since base < 1.0
                prop_assert!(us < base, "{us} >= {base}");
            }

            let approx_f64 = f64_approx(base).powf(exp as f64);
            let approx_uq0_63 = uq0_63_approx(approx_f64);

            // small error from f64 result
            let diff_u64 = us.0.abs_diff(approx_uq0_63.0);
            // same err bound as mul_pt since a.pow(2) = a * a
            prop_assert!(
                diff_u64 <= MAX_MUL_DIFF_F64_VS_US,
                "{}, {} {}",
                us.0,
                approx_f64,
                approx_uq0_63.0
            );

            // diff should not exceed epsilon proportion of value
            let diff_r = Ratio {
                n: diff_u64,
                d: min(us.0, approx_uq0_63.0),
            };
            prop_assert!(diff_r < EPSILON_RATIO_DIFF, "diff_r: {diff_r}");

            // pow of anything < 1.0 eventually reaches 0
            if base != UQ0_63::ONE {
                prop_assert_eq!(base.pow(u64::MAX), UQ0_63::ZERO);
            }

            // compare against naive multiplication implementation
            const LIM: u64 = u16::MAX as u64;
            let naive_mul_res = match exp {
                0 => UQ0_63::ONE,
                1..=LIM => (0..exp.saturating_sub(1)).fold(base, |res, _| res * base),
                _will_take_too_long_to_run => return Ok(())
            };
            // result will not be exactly eq bec each mult has rounding
            // and the 2 procedures mult differently
            let diff_r = Ratio {
                n: naive_mul_res.0.abs_diff(us.0),
                d: min(naive_mul_res.0, us.0),
            };
            prop_assert!(diff_r < EPSILON_RATIO_DIFF, "naive_mul diff_r: {diff_r}");
        }
    }

    // separate test from exp_pt bec strat doesnt seem to select boundary values
    // TODO: investigate. This doesnt seem like correct proptest behaviour
    proptest! {
        #[test]
        fn pow_zero_is_one(base in any_uq0_63_strat()) {
            prop_assert_eq!(base.pow(0), UQ0_63::ONE);
        }
    }

    // separate test from exp_pt bec strat doesnt seem to select boundary values
    // TODO: investigate. This doesnt seem like correct proptest behaviour
    proptest! {
        #[test]
        fn one_pow_is_one(exp: u64) {
            prop_assert_eq!(UQ0_63::ONE.pow(exp), UQ0_63::ONE);
        }
    }

    #[test]
    fn into_ratio_sc() {
        assert_eq!(UQ0_63(D / 2).into_ratio(), Ratio { n: 1, d: 2 });
    }

    #[test]
    fn one_mul_one_eq_one() {
        assert_eq!(UQ0_63::ONE * UQ0_63::ONE, UQ0_63::ONE);
    }

    #[test]
    fn uq0_63_new_sc() {
        const FAIL: u64 = D + 1;
        const SUCC: u64 = D;

        assert_eq!(UQ0_63::new(FAIL), Err(UQ0_63TooLargeErr { actual: FAIL }));
        assert_eq!(UQ0_63::new(SUCC), Ok(UQ0_63(SUCC)));
    }
}
