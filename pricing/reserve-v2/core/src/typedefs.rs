use core::{error::Error, fmt::Display, ops::Deref, slice};

use generic_array_struct::generic_array_struct;
use sanctum_fee_ratio::{
    ratio::{Ceil, Ratio},
    Fee,
};

use crate::internal_utils::{const_map, impl_cast_from_acc_data, impl_cast_to_acc_data};

pub const NANOS_DENOM: u32 = 1_000_000_000;

/// 100%
pub const MAX_FEE_NANOS: u32 = NANOS_DENOM;

/// Strictly greater than 0% so that band 1 (0% to threshold) always has positive width
pub const MIN_THRESHOLD_NANOS: u32 = 1;

/// Strictly less than 100% so that band 2 (threshold to 100%) always has positive width
pub const MAX_THRESHOLD_NANOS: u32 = NANOS_DENOM - 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Nanos(u32);

impl Nanos {
    #[inline]
    pub const fn get(&self) -> u32 {
        self.0
    }

    #[inline]
    pub const fn ratio(&self) -> Ratio<u32, u32> {
        Ratio {
            n: self.0,
            d: NANOS_DENOM,
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

/// Unsigned: negative fees are unsupported
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(transparent)]
pub struct FeeNanos(Nanos);

/// Constructors
impl FeeNanos {
    /// 0%
    pub const ZERO: Self = Self(Nanos(0));

    /// 100%
    pub const MAX: Self = Self(Nanos(MAX_FEE_NANOS));

    #[inline]
    pub const fn new(n: u32) -> Result<Self, FeeNanosOutOfRangeErr> {
        if n > MAX_FEE_NANOS {
            Err(FeeNanosOutOfRangeErr { actual: n })
        } else {
            Ok(Self(Nanos(n)))
        }
    }

    #[inline]
    pub const fn get(&self) -> u32 {
        self.0.get()
    }

    /// Retained rate in nanos: `NANOS_DENOM - fee`
    #[inline]
    pub const fn retained(&self) -> Self {
        Self(Nanos(NANOS_DENOM - self.0.get()))
    }

    #[inline]
    pub const fn retained_ratio(&self) -> Ratio<u32, u32> {
        // safety: FeeNanos is always <= NANOS_DENOM and denominator is nonzero
        unsafe {
            Fee::<Ceil<Ratio<u32, u32>>>::new_unchecked(Ratio {
                n: self.0.get(),
                d: NANOS_DENOM,
            })
        }
        .one_minus_fee_ratio()
    }
}

impl Deref for FeeNanos {
    type Target = Nanos;

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
pub struct ThresholdNanos(Nanos);

/// Constructors
impl ThresholdNanos {
    pub const MIN: Self = Self(Nanos(MIN_THRESHOLD_NANOS));

    pub const MAX: Self = Self(Nanos(MAX_THRESHOLD_NANOS));

    #[inline]
    pub const fn new(n: u32) -> Result<Self, ThresholdNanosOutOfRangeErr> {
        if n < MIN_THRESHOLD_NANOS || n > MAX_THRESHOLD_NANOS {
            Err(ThresholdNanosOutOfRangeErr { actual: n })
        } else {
            Ok(Self(Nanos(n)))
        }
    }

    #[inline]
    pub const fn get(&self) -> u32 {
        self.0.get()
    }

    #[inline]
    pub const fn ratio(&self) -> Ratio<u32, u32> {
        self.0.ratio()
    }
}

impl Deref for ThresholdNanos {
    type Target = Nanos;

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

#[generic_array_struct(all pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FeeEntryNanos<T> {
    pub base_fee: T,
    pub threshold_fee: T,
    pub max_fee: T,
    pub output_fee: T,
}

pub type FeeEntryNanosPacked = FeeEntryNanos<[u8; 4]>;
pub type FeeEntryNanosRaw = FeeEntryNanos<u32>;

/// # Invariants
/// - `fee_nanos` fields are valid [`FeeNanos`]
/// - `threshold_nanos` is a valid [`ThresholdNanos`]
/// - `base_fee <= threshold_fee <= max_fee`
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FeeEntryGen<M, T, F> {
    /// [`u8; 32`]
    pub mint: M,

    /// [`Nanos`]
    pub threshold_nanos: T,

    /// [`FeeEntryNanos`]
    pub fee_nanos: F,
}

pub type FeeEntry = FeeEntryGen<[u8; 32], u32, FeeEntryNanosRaw>;

impl FeeEntry {
    /// Relies on invariant: `threshold_nanos` is a valid [`ThresholdNanos`]
    #[inline]
    pub const fn threshold_nanos_typed(&self) -> ThresholdNanos {
        ThresholdNanos(Nanos(self.threshold_nanos))
    }

    /// Relies on invariant: `fee_nanos` fields are valid [`FeeNanos`]
    #[inline]
    pub const fn fee_nanos_typed(&self) -> FeeEntryNanos<FeeNanos> {
        FeeEntryNanos(const_map!(
            FeeNanos::ZERO,
            self.fee_nanos.0,
            u32_to_fee_nanos
        ))
    }
}

impl_cast_from_acc_data!(FeeEntry);
impl_cast_to_acc_data!(FeeEntry);

const _ASSERT_FEE_ENTRY_ALIGN: () = assert!(align_of::<FeeEntry>() == 4);

pub type FeeEntryPacked = FeeEntryGen<[u8; 32], [u8; 4], FeeEntryNanosPacked>;

impl FeeEntryPacked {
    #[inline]
    pub const fn mint(&self) -> &[u8; 32] {
        &self.mint
    }
}

impl FeeEntryPacked {
    #[inline]
    pub const fn into_fee_entry(self) -> FeeEntry {
        FeeEntry {
            mint: self.mint,
            threshold_nanos: le_bytes_to_u32(&self.threshold_nanos),
            fee_nanos: FeeEntryNanos(const_map!(0, self.fee_nanos.0, le_bytes_to_u32)),
        }
    }
}

impl FeeEntry {
    #[inline]
    pub const fn into_fee_entry_packed(self) -> FeeEntryPacked {
        FeeEntryPacked {
            mint: self.mint,
            threshold_nanos: u32_to_le_bytes(&self.threshold_nanos),
            fee_nanos: FeeEntryNanos(const_map!([0; 4], self.fee_nanos.0, u32_to_le_bytes)),
        }
    }
}

#[inline]
const fn le_bytes_to_u32(bytes: &[u8; 4]) -> u32 {
    u32::from_le_bytes(*bytes)
}

#[inline]
const fn u32_to_le_bytes(n: &u32) -> [u8; 4] {
    n.to_le_bytes()
}

#[inline]
const fn u32_to_fee_nanos(n: &u32) -> FeeNanos {
    FeeNanos(Nanos(*n))
}

impl From<FeeEntryPacked> for FeeEntry {
    #[inline]
    fn from(value: FeeEntryPacked) -> Self {
        value.into_fee_entry()
    }
}

impl From<FeeEntry> for FeeEntryPacked {
    #[inline]
    fn from(value: FeeEntry) -> Self {
        value.into_fee_entry_packed()
    }
}

impl_cast_from_acc_data!(FeeEntryPacked, packed);
impl_cast_to_acc_data!(FeeEntryPacked, packed);

const _ASSERT_FEE_ENTRY_PACKED_ALIGN: () = assert!(align_of::<FeeEntryPacked>() == 1);
const _ASSERT_FEE_ENTRY_PACKED_SIZE: () =
    assert!(size_of::<FeeEntryPacked>() == size_of::<FeeEntry>());
const _ASSERT_FEE_ENTRY_SIZE: () = assert!(size_of::<FeeEntry>() == 52);

/// Returns the number of fee entries if the account data length is valid.
const fn fee_entry_list_len(acc_data: &[u8]) -> Option<usize> {
    let tlen: usize = size_of::<FeeEntry>();
    #[allow(clippy::manual_is_multiple_of)]
    if acc_data.len() % tlen != 0 {
        return None;
    }
    Some(acc_data.len() / tlen)
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct FeeEntryList<'a>(pub &'a [FeeEntry]);

impl<'a> FeeEntryList<'a> {
    #[inline]
    pub const fn new(slice: &'a [FeeEntry]) -> Self {
        FeeEntryList(slice)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct FeeEntryPackedList<'a>(pub &'a [FeeEntryPacked]);

impl<'a> FeeEntryPackedList<'a> {
    #[inline]
    pub const fn new(slice: &'a [FeeEntryPacked]) -> Self {
        FeeEntryPackedList(slice)
    }
}

/// pointer casting "serde"
impl<'a> FeeEntryPackedList<'a> {
    #[inline]
    pub const fn of_acc_data(acc_data: &'a [u8]) -> Option<Self> {
        match fee_entry_list_len(acc_data) {
            None => None,
            // safety:
            // - FeeEntryPacked align == 1
            // - length is checked above by fee_entry_list_len
            Some(len) => Some(Self(unsafe {
                slice::from_raw_parts(acc_data.as_ptr().cast(), len)
            })),
        }
    }

    #[inline]
    pub const fn as_acc_data(&self) -> &[u8] {
        #[allow(clippy::manual_slice_size_calculation)]
        let bytes = self.0.len() * size_of::<FeeEntryPacked>();
        // safety: FeeEntryPacked has no internal padding
        unsafe { slice::from_raw_parts(self.0.as_ptr().cast(), bytes) }
    }
}

/// pointer casting "serde"
#[cfg(target_endian = "little")]
impl<'a> FeeEntryList<'a> {
    /// # Safety
    /// - `acc_data` must be aligned to `align_of::<FeeEntry>() == 4`
    /// - Solana guarantees this for on-chain account data
    /// - off-chain callers should use [`FeeEntryPackedList`] instead
    #[inline]
    pub const unsafe fn of_acc_data(acc_data: &'a [u8]) -> Option<Self> {
        let len = match fee_entry_list_len(acc_data) {
            None => return None,
            Some(len) => len,
        };
        // safety:
        // - caller guarantees acc_data is aligned to FeeEntry
        // - length is checked above by fee_entry_list_len
        // - FeeEntry has no internal padding
        Some(Self(unsafe {
            slice::from_raw_parts(acc_data.as_ptr().cast(), len)
        }))
    }

    #[inline]
    pub fn as_acc_data(&self) -> &[u8] {
        #[allow(clippy::manual_slice_size_calculation)]
        let bytes = self.0.len() * size_of::<FeeEntry>();
        // safety: FeeEntry has no internal padding
        unsafe { slice::from_raw_parts(self.0.as_ptr().cast(), bytes) }
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct FeeEntryListMut<'a>(pub &'a mut [FeeEntry]);

/// pointer casting "deserialization"
#[cfg(target_endian = "little")]
impl<'a> FeeEntryListMut<'a> {
    /// # Safety
    /// - `acc_data` must be aligned to `align_of::<FeeEntry>() == 4`
    /// - Solana guarantees this for on-chain account data
    #[inline]
    pub unsafe fn of_acc_data(acc_data: &'a mut [u8]) -> Option<Self> {
        let len = fee_entry_list_len(acc_data)?;
        // safety:
        // - caller guarantees acc_data is aligned to FeeEntry
        // - length is checked above by fee_entry_list_len
        // - FeeEntry has no internal padding
        Some(Self(unsafe {
            slice::from_raw_parts_mut(acc_data.as_mut_ptr().cast(), len)
        }))
    }
}

/// to immut
impl FeeEntryListMut<'_> {
    #[inline]
    pub const fn as_list(&self) -> FeeEntryList<'_> {
        FeeEntryList(self.0)
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
impl FeeEntryList<'_> {
    #[inline]
    pub fn find_idx_by_mint(&self, mint: &[u8; 32]) -> Result<usize, MintNotFoundErr> {
        self.0
            .binary_search_by_key(mint, |entry| entry.mint)
            .map_err(|expected_i| MintNotFoundErr {
                expected_i,
                mint: *mint,
            })
    }

    #[inline]
    pub fn find_by_mint(&self, mint: &[u8; 32]) -> Result<&FeeEntry, MintNotFoundErr> {
        self.find_idx_by_mint(mint).map(|i| &self.0[i])
    }
}

impl FeeEntryListMut<'_> {
    #[inline]
    pub fn find_by_mint_mut(&mut self, mint: &[u8; 32]) -> Result<&mut FeeEntry, MintNotFoundErr> {
        self.as_list()
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
