use core::{error::Error, fmt::Display, ops::Deref, slice};

use crate::internal_utils::{impl_cast_from_acc_data, impl_cast_to_acc_data};

pub const NANOS_DENOM: u32 = 1_000_000_000;

/// 100%
pub const MAX_FEE_NANOS: u32 = NANOS_DENOM;

/// Strictly greater than 0% so that band 1 (0% to threshold) always has positive width
pub const MIN_THRESHOLD_NANOS: u32 = 1;

/// Strictly less than 100% so that band 2 (threshold to 100%) always has positive width
pub const MAX_THRESHOLD_NANOS: u32 = NANOS_DENOM - 1;

/// Unsigned: negative fees are unsupported
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(transparent)]
pub struct FeeNanos(u32);

/// Constructors
impl FeeNanos {
    /// 0%
    pub const ZERO: Self = Self(0);

    /// 100%
    pub const MAX: Self = Self(MAX_FEE_NANOS);

    #[inline]
    pub const fn new(n: u32) -> Result<Self, FeeNanosOutOfRangeErr> {
        if n > MAX_FEE_NANOS {
            Err(FeeNanosOutOfRangeErr { actual: n })
        } else {
            Ok(Self(n))
        }
    }

    #[inline]
    pub const fn get(&self) -> u32 {
        self.0
    }

    /// Retained rate in nanos: `NANOS_DENOM - fee`
    #[inline]
    pub const fn retained(&self) -> u32 {
        NANOS_DENOM - self.0
    }
}

impl Deref for FeeNanos {
    type Target = u32;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FeeNanosOutOfRangeErr {
    pub actual: u32,
}

impl Display for FeeNanosOutOfRangeErr {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let Self { actual } = self;
        f.write_fmt(format_args!("fee nanos {actual} > {MAX_FEE_NANOS} (max)"))
    }
}

impl Error for FeeNanosOutOfRangeErr {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(transparent)]
pub struct ThresholdNanos(u32);

/// Constructors
impl ThresholdNanos {
    pub const MIN: Self = Self(MIN_THRESHOLD_NANOS);

    pub const MAX: Self = Self(MAX_THRESHOLD_NANOS);

    #[inline]
    pub const fn new(n: u32) -> Result<Self, ThresholdNanosOutOfRangeErr> {
        if n < MIN_THRESHOLD_NANOS || n > MAX_THRESHOLD_NANOS {
            Err(ThresholdNanosOutOfRangeErr { actual: n })
        } else {
            Ok(Self(n))
        }
    }

    #[inline]
    pub const fn get(&self) -> u32 {
        self.0
    }
}

impl Deref for ThresholdNanos {
    type Target = u32;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ThresholdNanosOutOfRangeErr {
    pub actual: u32,
}

impl Display for ThresholdNanosOutOfRangeErr {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let Self { actual } = self;
        if self.actual > MAX_THRESHOLD_NANOS {
            f.write_fmt(format_args!(
                "threshold nanos {actual} > {MAX_THRESHOLD_NANOS} (max)"
            ))
        } else {
            f.write_fmt(format_args!(
                "threshold nanos {actual} < {MIN_THRESHOLD_NANOS} (min)"
            ))
        }
    }
}

impl Error for ThresholdNanosOutOfRangeErr {}

/// # Invariants
/// - fee fields are valid [`FeeNanos`]
/// - `threshold_nanos` is a valid [`ThresholdNanos`]
/// - `base_fee_nanos <= threshold_fee_nanos <= max_fee_nanos`
///
/// Established by [`Self::new`] and re-established by [`Self::validate`] at
/// write boundaries. Callers that mutate in place must [`Self::validate`]
/// before persisting.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FeeEntryPacked {
    pub(crate) mint: [u8; 32],
    pub(crate) base_fee_nanos: [u8; 4],
    pub(crate) threshold_nanos: [u8; 4],
    pub(crate) threshold_fee_nanos: [u8; 4],
    pub(crate) max_fee_nanos: [u8; 4],
    pub(crate) output_fee_nanos: [u8; 4],
}

/// Constructors
impl FeeEntryPacked {
    #[inline]
    pub const fn new(
        mint: [u8; 32],
        base_fee_nanos: FeeNanos,
        threshold_nanos: ThresholdNanos,
        threshold_fee_nanos: FeeNanos,
        max_fee_nanos: FeeNanos,
        output_fee_nanos: FeeNanos,
    ) -> Result<Self, InvalidFeeEntryErr> {
        if base_fee_nanos.get() > threshold_fee_nanos.get()
            || threshold_fee_nanos.get() > max_fee_nanos.get()
        {
            return Err(InvalidFeeEntryErr::NonMonotoneFees {
                base_fee_nanos: base_fee_nanos.get(),
                threshold_fee_nanos: threshold_fee_nanos.get(),
                max_fee_nanos: max_fee_nanos.get(),
            });
        }
        Ok(Self {
            mint,
            base_fee_nanos: base_fee_nanos.get().to_le_bytes(),
            threshold_nanos: threshold_nanos.get().to_le_bytes(),
            threshold_fee_nanos: threshold_fee_nanos.get().to_le_bytes(),
            max_fee_nanos: max_fee_nanos.get().to_le_bytes(),
            output_fee_nanos: output_fee_nanos.get().to_le_bytes(),
        })
    }
}

/// Accessors
impl FeeEntryPacked {
    #[inline]
    pub const fn mint(&self) -> &[u8; 32] {
        &self.mint
    }

    #[inline]
    pub const fn base_fee_nanos(&self) -> FeeNanos {
        FeeNanos(u32::from_le_bytes(self.base_fee_nanos))
    }

    #[inline]
    pub const fn threshold_nanos(&self) -> ThresholdNanos {
        ThresholdNanos(u32::from_le_bytes(self.threshold_nanos))
    }

    #[inline]
    pub const fn threshold_fee_nanos(&self) -> FeeNanos {
        FeeNanos(u32::from_le_bytes(self.threshold_fee_nanos))
    }

    #[inline]
    pub const fn max_fee_nanos(&self) -> FeeNanos {
        FeeNanos(u32::from_le_bytes(self.max_fee_nanos))
    }

    #[inline]
    pub const fn output_fee_nanos(&self) -> FeeNanos {
        FeeNanos(u32::from_le_bytes(self.output_fee_nanos))
    }
}

/// Validation
impl FeeEntryPacked {
    #[inline]
    pub const fn validate(&self) -> Result<(), InvalidFeeEntryErr> {
        let base_fee_nanos = match FeeNanos::new(u32::from_le_bytes(self.base_fee_nanos)) {
            Ok(fee_nanos) => fee_nanos,
            Err(e) => return Err(InvalidFeeEntryErr::BaseFeeOutOfRange(e)),
        };

        if let Err(e) = ThresholdNanos::new(u32::from_le_bytes(self.threshold_nanos)) {
            return Err(InvalidFeeEntryErr::ThresholdOutOfRange(e));
        }

        let threshold_fee_nanos = match FeeNanos::new(u32::from_le_bytes(self.threshold_fee_nanos))
        {
            Ok(fee_nanos) => fee_nanos,
            Err(e) => return Err(InvalidFeeEntryErr::ThresholdFeeOutOfRange(e)),
        };

        let max_fee_nanos = match FeeNanos::new(u32::from_le_bytes(self.max_fee_nanos)) {
            Ok(fee_nanos) => fee_nanos,
            Err(e) => return Err(InvalidFeeEntryErr::MaxFeeOutOfRange(e)),
        };

        if let Err(e) = FeeNanos::new(u32::from_le_bytes(self.output_fee_nanos)) {
            return Err(InvalidFeeEntryErr::OutputFeeOutOfRange(e));
        }

        let base_fee_nanos = base_fee_nanos.get();
        let threshold_fee_nanos = threshold_fee_nanos.get();
        let max_fee_nanos = max_fee_nanos.get();
        if base_fee_nanos > threshold_fee_nanos || threshold_fee_nanos > max_fee_nanos {
            return Err(InvalidFeeEntryErr::NonMonotoneFees {
                base_fee_nanos,
                threshold_fee_nanos,
                max_fee_nanos,
            });
        }

        Ok(())
    }
}

/// Mutators
impl FeeEntryPacked {
    #[inline]
    pub const fn mint_mut(&mut self) -> &mut [u8; 32] {
        &mut self.mint
    }

    #[inline]
    pub const fn set_base_fee_nanos(&mut self, base_fee_nanos: FeeNanos) {
        self.base_fee_nanos = base_fee_nanos.get().to_le_bytes();
    }

    #[inline]
    pub const fn set_threshold_nanos(&mut self, threshold_nanos: ThresholdNanos) {
        self.threshold_nanos = threshold_nanos.get().to_le_bytes();
    }

    #[inline]
    pub const fn set_threshold_fee_nanos(&mut self, threshold_fee_nanos: FeeNanos) {
        self.threshold_fee_nanos = threshold_fee_nanos.get().to_le_bytes();
    }

    #[inline]
    pub const fn set_max_fee_nanos(&mut self, max_fee_nanos: FeeNanos) {
        self.max_fee_nanos = max_fee_nanos.get().to_le_bytes();
    }

    #[inline]
    pub const fn set_output_fee_nanos(&mut self, output_fee_nanos: FeeNanos) {
        self.output_fee_nanos = output_fee_nanos.get().to_le_bytes();
    }
}

impl_cast_from_acc_data!(FeeEntryPacked, packed);
impl_cast_to_acc_data!(FeeEntryPacked, packed);

const _ASSERT_FEE_ENTRY_PACKED_ALIGN: () = assert!(align_of::<FeeEntryPacked>() == 1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InvalidFeeEntryErr {
    BaseFeeOutOfRange(FeeNanosOutOfRangeErr),
    ThresholdOutOfRange(ThresholdNanosOutOfRangeErr),
    ThresholdFeeOutOfRange(FeeNanosOutOfRangeErr),
    MaxFeeOutOfRange(FeeNanosOutOfRangeErr),
    OutputFeeOutOfRange(FeeNanosOutOfRangeErr),
    NonMonotoneFees {
        base_fee_nanos: u32,
        threshold_fee_nanos: u32,
        max_fee_nanos: u32,
    },
}

impl Display for InvalidFeeEntryErr {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::BaseFeeOutOfRange(e) => {
                f.write_fmt(format_args!("base fee out of range: {e}"))
            }
            Self::ThresholdOutOfRange(e) => {
                f.write_fmt(format_args!("threshold out of range: {e}"))
            }
            Self::ThresholdFeeOutOfRange(e) => {
                f.write_fmt(format_args!("threshold fee out of range: {e}"))
            }
            Self::MaxFeeOutOfRange(e) => {
                f.write_fmt(format_args!("max fee out of range: {e}"))
            }
            Self::OutputFeeOutOfRange(e) => {
                f.write_fmt(format_args!("output fee out of range: {e}"))
            }
            Self::NonMonotoneFees {
                base_fee_nanos,
                threshold_fee_nanos,
                max_fee_nanos,
            } => f.write_fmt(format_args!(
                "non-monotone fees: base {base_fee_nanos}, threshold {threshold_fee_nanos}, max {max_fee_nanos}"
            )),
        }
    }
}

impl Error for InvalidFeeEntryErr {}

/// Returns element length of [`PackedList`] if acc_data is a valid one
const fn packed_list_len<T>(acc_data: &[u8]) -> Option<usize> {
    const {
        assert!(align_of::<T>() == 1);
    }

    let tlen: usize = size_of::<T>();
    #[allow(clippy::manual_is_multiple_of)]
    if acc_data.len() % tlen != 0 {
        return None;
    }
    Some(acc_data.len() / tlen)
}

pub type FeeEntryPackedList<'a> = PackedList<'a, FeeEntryPacked>;
pub type FeeEntryPackedListMut<'a> = PackedListMut<'a, FeeEntryPacked>;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct PackedList<'a, T>(pub &'a [T]);

impl<'a, T> PackedList<'a, T> {
    /// For more convenient usage with type aliases
    #[inline]
    pub const fn new(slice: &'a [T]) -> Self {
        PackedList(slice)
    }
}

/// pointer casting "serde"
impl<'a, T> PackedList<'a, T> {
    #[inline]
    pub const fn of_acc_data(acc_data: &'a [u8]) -> Option<Self> {
        match packed_list_len::<T>(acc_data) {
            None => None,
            Some(len) => Some(Self(unsafe {
                slice::from_raw_parts(acc_data.as_ptr().cast(), len)
            })),
        }
    }

    #[inline]
    pub const fn as_acc_data(&self) -> &[u8] {
        #[allow(clippy::manual_slice_size_calculation)]
        let bytes = self.0.len() * size_of::<T>();
        unsafe { slice::from_raw_parts(self.0.as_ptr().cast(), bytes) }
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct PackedListMut<'a, T>(pub &'a mut [T]);

/// pointer casting "deserialization"
impl<'a, T> PackedListMut<'a, T> {
    #[inline]
    pub const fn of_acc_data(acc_data: &'a mut [u8]) -> Option<Self> {
        match packed_list_len::<T>(acc_data) {
            None => None,
            Some(len) => Some(Self(unsafe {
                slice::from_raw_parts_mut(acc_data.as_mut_ptr().cast(), len)
            })),
        }
    }
}

/// to immut
impl<T> PackedListMut<'_, T> {
    #[inline]
    pub const fn as_packed_list(&self) -> PackedList<'_, T> {
        PackedList(self.0)
    }
}

/// Accessors
impl FeeEntryPackedList<'_> {
    #[inline]
    pub fn find_idx_by_mint(&self, mint: &[u8; 32]) -> Result<usize, MintNotFoundErr> {
        self.0
            .binary_search_by_key(mint, |entry| *entry.mint())
            .map_err(|expected_i| MintNotFoundErr {
                expected_i,
                mint: *mint,
            })
    }

    #[inline]
    pub fn find_by_mint(&self, mint: &[u8; 32]) -> Result<&FeeEntryPacked, MintNotFoundErr> {
        self.find_idx_by_mint(mint).map(|i| &self.0[i])
    }
}

/// Accessors
impl FeeEntryPackedListMut<'_> {
    #[inline]
    pub fn find_by_mint_mut(
        &mut self,
        mint: &[u8; 32],
    ) -> Result<&mut FeeEntryPacked, MintNotFoundErr> {
        self.as_packed_list()
            .find_idx_by_mint(mint)
            .map(|i| &mut self.0[i])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MintNotFoundErr {
    /// index to insert this mint at to maintain sorted order
    pub expected_i: usize,
    pub mint: [u8; 32],
}

impl Display for MintNotFoundErr {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("MintNotFound")
    }
}

impl Error for MintNotFoundErr {}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_entry(mint: [u8; 32]) -> FeeEntryPacked {
        FeeEntryPacked::new(
            mint,
            FeeNanos::new(1).unwrap(),
            ThresholdNanos::new(2).unwrap(),
            FeeNanos::new(3).unwrap(),
            FeeNanos::new(4).unwrap(),
            FeeNanos::new(5).unwrap(),
        )
        .unwrap()
    }

    #[test]
    fn fee_entry_new_accepts_equal_fees() {
        let entry = FeeEntryPacked::new(
            [0; 32],
            FeeNanos::ZERO,
            ThresholdNanos::MIN,
            FeeNanos::ZERO,
            FeeNanos::ZERO,
            FeeNanos::ZERO,
        )
        .unwrap();
        assert_eq!(entry.validate(), Ok(()));
    }

    #[test]
    fn fee_entry_new_rejects_non_monotone() {
        // base > threshold_fee
        assert_eq!(
            FeeEntryPacked::new(
                [0; 32],
                FeeNanos::new(2).unwrap(),
                ThresholdNanos::MIN,
                FeeNanos::new(1).unwrap(),
                FeeNanos::MAX,
                FeeNanos::ZERO,
            ),
            Err(InvalidFeeEntryErr::NonMonotoneFees {
                base_fee_nanos: 2,
                threshold_fee_nanos: 1,
                max_fee_nanos: MAX_FEE_NANOS,
            })
        );
        // threshold_fee > max
        assert_eq!(
            FeeEntryPacked::new(
                [0; 32],
                FeeNanos::ZERO,
                ThresholdNanos::MIN,
                FeeNanos::new(2).unwrap(),
                FeeNanos::new(1).unwrap(),
                FeeNanos::ZERO,
            ),
            Err(InvalidFeeEntryErr::NonMonotoneFees {
                base_fee_nanos: 0,
                threshold_fee_nanos: 2,
                max_fee_nanos: 1,
            })
        );
    }

    #[test]
    fn fee_entry_validate_rejection_matrix() {
        const OVER: u32 = MAX_FEE_NANOS + 1;

        let mut entry = valid_entry([0; 32]);
        entry.base_fee_nanos = OVER.to_le_bytes();
        assert_eq!(
            entry.validate(),
            Err(InvalidFeeEntryErr::BaseFeeOutOfRange(
                FeeNanosOutOfRangeErr { actual: OVER }
            ))
        );

        let mut entry = valid_entry([0; 32]);
        entry.threshold_nanos = 0_u32.to_le_bytes();
        assert_eq!(
            entry.validate(),
            Err(InvalidFeeEntryErr::ThresholdOutOfRange(
                ThresholdNanosOutOfRangeErr { actual: 0 }
            ))
        );
        entry.threshold_nanos = NANOS_DENOM.to_le_bytes();
        assert_eq!(
            entry.validate(),
            Err(InvalidFeeEntryErr::ThresholdOutOfRange(
                ThresholdNanosOutOfRangeErr {
                    actual: NANOS_DENOM
                }
            ))
        );

        let mut entry = valid_entry([0; 32]);
        entry.threshold_fee_nanos = OVER.to_le_bytes();
        assert_eq!(
            entry.validate(),
            Err(InvalidFeeEntryErr::ThresholdFeeOutOfRange(
                FeeNanosOutOfRangeErr { actual: OVER }
            ))
        );

        let mut entry = valid_entry([0; 32]);
        entry.max_fee_nanos = OVER.to_le_bytes();
        assert_eq!(
            entry.validate(),
            Err(InvalidFeeEntryErr::MaxFeeOutOfRange(
                FeeNanosOutOfRangeErr { actual: OVER }
            ))
        );

        let mut entry = valid_entry([0; 32]);
        entry.output_fee_nanos = OVER.to_le_bytes();
        assert_eq!(
            entry.validate(),
            Err(InvalidFeeEntryErr::OutputFeeOutOfRange(
                FeeNanosOutOfRangeErr { actual: OVER }
            ))
        );

        // setters can transiently violate monotonicity, validate catches it
        let mut entry = valid_entry([0; 32]);
        entry.set_base_fee_nanos(FeeNanos::MAX);
        assert_eq!(
            entry.validate(),
            Err(InvalidFeeEntryErr::NonMonotoneFees {
                base_fee_nanos: MAX_FEE_NANOS,
                threshold_fee_nanos: 3,
                max_fee_nanos: 4,
            })
        );
    }
}
