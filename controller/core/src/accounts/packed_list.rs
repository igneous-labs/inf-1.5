use core::{
    mem::{align_of, size_of},
    slice,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PackedList<'a, T>(pub &'a [T]);

/// Given the length in bytes of a packed list account `PackedList<T>`,
/// return the number of elems in it.
///
/// Returns `None` if given byte_len is not a valid length for
/// a PackedList of the given type `T`
///
/// Basically just `byte_len / size_of::<T>()`
#[inline]
pub const fn packed_list_len<T>(byte_len: usize) -> Option<usize> {
    // cant use a const here due to generic being outer
    let tlen: usize = size_of::<T>();
    // is_multiple_of doesnt exist in rustc 1.84
    #[allow(clippy::manual_is_multiple_of)]
    if byte_len % tlen != 0 {
        return None;
    }
    Some(byte_len / tlen)
}

/// pointer casting "serde"
impl<'a, T> PackedList<'a, T> {
    #[inline]
    const fn of_acc_data_inner(acc_data: &'a [u8]) -> Option<Self> {
        let len = match packed_list_len::<T>(acc_data.len()) {
            None => return None,
            Some(x) => x,
        };
        Some(Self(unsafe {
            slice::from_raw_parts(acc_data.as_ptr().cast(), len)
        }))
    }

    #[inline]
    pub const fn of_acc_data(acc_data: &'a [u8]) -> Option<Self> {
        const {
            assert!(align_of::<T>() == 1);
        }
        Self::of_acc_data_inner(acc_data)
    }

    /// # Safety
    /// - `acc_data` must have the same align as `T`
    #[inline]
    pub const unsafe fn of_acc_data_unsafe(acc_data: &'a [u8]) -> Option<Self> {
        Self::of_acc_data_inner(acc_data)
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

/// pointer casting "serde"
impl<'a, T> PackedListMut<'a, T> {
    #[inline]
    const fn of_acc_data_inner(acc_data: &'a mut [u8]) -> Option<Self> {
        let len = match packed_list_len::<T>(acc_data.len()) {
            None => return None,
            Some(x) => x,
        };
        Some(Self(unsafe {
            slice::from_raw_parts_mut(acc_data.as_mut_ptr().cast(), len)
        }))
    }

    #[inline]
    pub const fn of_acc_data(acc_data: &'a mut [u8]) -> Option<Self> {
        const {
            assert!(align_of::<T>() == 1);
        }
        Self::of_acc_data_inner(acc_data)
    }

    /// # Safety
    /// - same requirements as [`PackedList::of_acc_data_unsafe`]
    #[inline]
    pub const unsafe fn of_acc_data_unsafe(acc_data: &'a mut [u8]) -> Option<Self> {
        Self::of_acc_data_inner(acc_data)
    }

    #[inline]
    pub const fn as_packed_list(&self) -> PackedList<'_, T> {
        PackedList(self.0)
    }
}
