#![cfg_attr(not(test), no_std)]

use core::ops::RangeInclusive;

use sanctum_fee_ratio::Fee;
use sanctum_u64_ratio::{Ceil, Floor, Ratio};

// Reference:
// https://github.com/igneous-labs/sanctum-solana-utils/blob/7066356530a09a32f9068aba30c75f8160e82cd6/sanctum-token-ratio/src/u64_ratio/floor.rs#L21-L79
#[inline]
pub const fn floor_ratio_u64_u64_reverse(
    ratio: Floor<Ratio<u64, u64>>,
    amt_after_apply: u64,
) -> Option<RangeInclusive<u64>> {
    if ratio.0.is_zero() {
        return if amt_after_apply == 0 {
            Some(0..=u64::MAX)
        } else {
            None
        };
    }

    let Ratio { n, d } = ratio.0;
    let d = d as u128;
    let n = n as u128;
    let y = amt_after_apply as u128;

    // unchecked-arith: mul will not overflow because
    // both d and y are <= u64::MAX
    let dy = d * y;
    // unchecked-arith: ratio is not 0 so n != 0
    let min = dy / n;
    let min = match u128_to_u64_checked(min) {
        None => return None,
        Some(r) => r,
    };

    // unchecked-arith: even if d = y = u64::MAX,
    // does not overflow u128
    // (2^64 - 1)^2 + 2^64 - 1
    // = 2^128 - 2^65 + 1 + 2^64 - 1
    // = 2^128 - 2^64(2 - 1)
    // = 2^128 - 2^64
    // < 2^128 - 1
    // = u128::MAX
    let dy_plus_d = dy + d;
    // unchecked-arith: ratio is not 0 so n != 0
    let max = dy_plus_d.div_ceil(n);
    let max = match u128_to_u64_checked(max) {
        None => return None,
        Some(r) => r,
    };

    Some(min..=max)
}

// Reference:
// https://github.com/igneous-labs/sanctum-solana-utils/blob/7066356530a09a32f9068aba30c75f8160e82cd6/sanctum-token-ratio/src/u64_fee_ratio/ceil.rs#L19-L55
#[inline]
pub const fn fee_ceil_ratio_u64_u64_reverse_from_rem(
    fee: Fee<Ceil<Ratio<u64, u64>>>,
    rem: u64,
) -> Option<RangeInclusive<u64>> {
    if fee.as_inner_ref().0.is_zero() {
        Some(rem..=rem)
    } else {
        floor_ratio_u64_u64_reverse(Floor(fee.one_minus_fee_ratio()), rem)
    }
}

// Reference:
// https://github.com/igneous-labs/sanctum-solana-utils/blob/7066356530a09a32f9068aba30c75f8160e82cd6/sanctum-token-ratio/src/u64_ratio/ceil.rs#L21-L86
#[inline]
pub const fn ceil_ratio_u32_u32_reverse(
    ratio: Ceil<Ratio<u32, u32>>,
    amt_after_apply: u64,
) -> Option<RangeInclusive<u64>> {
    if ratio.0.is_zero() {
        return if amt_after_apply == 0 {
            Some(0..=u64::MAX)
        } else {
            None
        };
    }
    // only way to get 0 after ceil div by a non-zero ratio is if input was 0.
    // early return ensures dy - d below does not overflow
    if amt_after_apply == 0 {
        return Some(0..=0);
    }

    let Ratio { n, d } = ratio.0;
    let d = d as u128;
    let n = n as u128;
    let y = amt_after_apply as u128;

    // unchecked-arith: mul will not overflow because
    // both d and y are <= u64::MAX
    let dy = d * y;

    // unchecked-arith: dy >= d
    let dy_minus_d = dy - d;

    // unchecked-arith: ratio is not 0 so n != 0
    let min = dy_minus_d / n;
    let min = match u128_to_u64_checked(min) {
        None => return None,
        Some(r) => r,
    };

    // unchecked-arith: ratio is not 0 so n != 0
    let max = dy.div_ceil(n);
    let max = match u128_to_u64_checked(max) {
        None => return None,
        Some(r) => r,
    };

    Some(min..=max)
}

// Reference:
// https://github.com/igneous-labs/sanctum-solana-utils/blob/7066356530a09a32f9068aba30c75f8160e82cd6/sanctum-token-ratio/src/u64_fee_ratio/floor.rs#L19-L55
#[inline]
pub const fn fee_floor_ratio_u32_u32_reverse_from_rem(
    fee: Fee<Floor<Ratio<u32, u32>>>,
    rem: u64,
) -> Option<RangeInclusive<u64>> {
    if fee.as_inner_ref().0.is_zero() {
        Some(rem..=rem)
    } else {
        ceil_ratio_u32_u32_reverse(Ceil(fee.one_minus_fee_ratio()), rem)
    }
}

#[inline]
const fn u128_to_u64_checked(x: u128) -> Option<u64> {
    if x > u64::MAX as u128 {
        None
    } else {
        Some(x as u64)
    }
}
