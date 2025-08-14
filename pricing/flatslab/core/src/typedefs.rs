use core::slice;

use crate::internal_utils::{impl_cast_from_acc_data, impl_cast_to_acc_data};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SlabEntryPacked {
    mint: [u8; 32],
    inp_fee_nanos: [u8; 4],
    out_fee_nanos: [u8; 4],
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
    pub const fn inp_fee_nanos(&self) -> i32 {
        i32::from_le_bytes(self.inp_fee_nanos)
    }

    #[inline]
    pub const fn out_fee_nanos(&self) -> i32 {
        i32::from_le_bytes(self.out_fee_nanos)
    }
}

/// Mutators
impl SlabEntryPacked {
    #[inline]
    pub const fn mint_mut(&mut self) -> &mut [u8; 32] {
        &mut self.mint
    }

    #[inline]
    pub const fn set_inp_fee_nanos(&mut self, inp_fee_nanos: i32) {
        self.inp_fee_nanos = inp_fee_nanos.to_le_bytes();
    }

    #[inline]
    pub const fn set_out_fee_nanos(&mut self, out_fee_nanos: i32) {
        self.out_fee_nanos = out_fee_nanos.to_le_bytes();
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
    if acc_data.len() % tlen != 0 {
        return None;
    }
    Some(acc_data.len() / tlen)
}

pub type SlabEntryPackedList<'a> = PackedList<'a, SlabEntryPacked>;
pub type SlabEntryPackedListMut<'a> = PackedListMut<'a, SlabEntryPacked>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PackedList<'a, T>(pub &'a [T]);

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

/// Accssors
impl SlabEntryPackedList<'_> {
    /// Returns `Err(index to insert to maintain sorted order)` if entry of mint not in list
    #[inline]
    pub fn find_by_mint(&self, mint: &[u8; 32]) -> Result<&SlabEntryPacked, usize> {
        self.0
            .binary_search_by_key(mint, |entry| *entry.mint())
            .map(|i| &self.0[i])
    }
}

/// Accessors
impl SlabEntryPackedListMut<'_> {
    /// Returns `Err(index to insert to maintain sorted order)` if entry of mint not in list
    #[inline]
    pub fn find_by_mint_mut(&mut self, mint: &[u8; 32]) -> Result<&mut SlabEntryPacked, usize> {
        self.0
            .binary_search_by_key(mint, |entry| *entry.mint())
            .map(|i| &mut self.0[i])
    }
}
