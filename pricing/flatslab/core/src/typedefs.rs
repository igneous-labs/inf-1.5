use core::{error::Error, fmt::Display, ops::Deref, slice};

use inf1_pp_core::pair::Pair;

use crate::{
    internal_utils::{impl_cast_from_acc_data, impl_cast_to_acc_data},
    pricing::FlatSlabSwapPricing,
};

pub const NANOS_DENOM: i32 = 1_000_000_000;

/// -5%
pub const MIN_FEE_NANOS: i32 = -50_000_000;

/// 100%
pub const MAX_FEE_NANOS: i32 = NANOS_DENOM;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(transparent)]
pub struct FeeNanos(i32);

/// Constructors
impl FeeNanos {
    /// -5%
    pub const MIN: Self = Self(MIN_FEE_NANOS);

    /// 100%
    pub const MAX: Self = Self(MAX_FEE_NANOS);

    #[inline]
    pub const fn new(n: i32) -> Result<Self, FeeNanosOutOfRangeErr> {
        if n > MAX_FEE_NANOS || n < MIN_FEE_NANOS {
            Err(FeeNanosOutOfRangeErr { actual: n })
        } else {
            Ok(Self(n))
        }
    }

    #[inline]
    pub const fn get(&self) -> i32 {
        self.0
    }
}

impl Deref for FeeNanos {
    type Target = i32;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FeeNanosOutOfRangeErr {
    pub actual: i32,
}

impl Display for FeeNanosOutOfRangeErr {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let Self { actual } = self;
        if self.actual > MAX_FEE_NANOS {
            f.write_fmt(format_args!("fee nanos {actual} > {MAX_FEE_NANOS} (max)"))
        } else {
            f.write_fmt(format_args!("fee nanos {actual} < {MIN_FEE_NANOS} (min)"))
        }
    }
}

impl Error for FeeNanosOutOfRangeErr {}

/// # Invariants
/// - `inp_fee_nanos` and `out_fee_nanos` must be of valid [`FeeNanos`] (in range)
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SlabEntryPacked {
    pub(crate) mint: [u8; 32],
    pub(crate) inp_fee_nanos: [u8; 4],
    pub(crate) out_fee_nanos: [u8; 4],
}

/// Constructors
impl SlabEntryPacked {
    pub const DEFAULT: Self = Self {
        mint: [0; 32],
        inp_fee_nanos: [0; 4],
        out_fee_nanos: [0; 4],
    };
}

impl Default for SlabEntryPacked {
    #[inline]
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// Accessors
impl SlabEntryPacked {
    #[inline]
    pub const fn mint(&self) -> &[u8; 32] {
        &self.mint
    }

    #[inline]
    pub const fn inp_fee_nanos(&self) -> FeeNanos {
        FeeNanos(i32::from_le_bytes(self.inp_fee_nanos))
    }

    #[inline]
    pub const fn out_fee_nanos(&self) -> FeeNanos {
        FeeNanos(i32::from_le_bytes(self.out_fee_nanos))
    }
}

/// Mutators
impl SlabEntryPacked {
    #[inline]
    pub const fn mint_mut(&mut self) -> &mut [u8; 32] {
        &mut self.mint
    }

    #[inline]
    pub const fn set_inp_fee_nanos(&mut self, inp_fee_nanos: FeeNanos) {
        self.inp_fee_nanos = inp_fee_nanos.get().to_le_bytes();
    }

    #[inline]
    pub const fn set_out_fee_nanos(&mut self, out_fee_nanos: FeeNanos) {
        self.out_fee_nanos = out_fee_nanos.get().to_le_bytes();
    }
}

impl_cast_from_acc_data!(SlabEntryPacked, packed);
impl_cast_to_acc_data!(SlabEntryPacked, packed);

const _ASSERT_SLAB_ENTRY_PACKED_ALIGN: () = assert!(align_of::<SlabEntryPacked>() == 1);

/// Returns element length of [`PackedList`] if acc_data is a valid one
const fn packed_list_len<T>(acc_data: &[u8]) -> Option<usize> {
    const {
        assert!(align_of::<T>() == 1);
    }

    let tlen: usize = size_of::<T>();
    // is_multiple_of doesnt exist in rustc 1.84
    #[allow(clippy::manual_is_multiple_of)]
    if acc_data.len() % tlen != 0 {
        return None;
    }
    Some(acc_data.len() / tlen)
}

pub type SlabEntryPackedList<'a> = PackedList<'a, SlabEntryPacked>;
pub type SlabEntryPackedListMut<'a> = PackedListMut<'a, SlabEntryPacked>;

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
        // core::mem::size_of_val not yet const in rustc 1.84
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
impl SlabEntryPackedList<'_> {
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
    pub fn find_by_mint(&self, mint: &[u8; 32]) -> Result<&SlabEntryPacked, MintNotFoundErr> {
        self.find_idx_by_mint(mint).map(|i| &self.0[i])
    }

    #[inline]
    pub fn pricing(&self, mints: &Pair<&[u8; 32]>) -> Result<FlatSlabSwapPricing, MintNotFoundErr> {
        let Pair { inp, out } = mints.try_map(|m| self.find_by_mint(m))?;
        Ok(FlatSlabSwapPricing {
            inp_fee_nanos: inp.inp_fee_nanos(),
            out_fee_nanos: out.out_fee_nanos(),
        })
    }
}

/// Accessors
impl SlabEntryPackedListMut<'_> {
    #[inline]
    pub fn find_by_mint_mut(
        &mut self,
        mint: &[u8; 32],
    ) -> Result<&mut SlabEntryPacked, MintNotFoundErr> {
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
