use core::{
    mem::{size_of, size_of_val},
    slice,
};

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct PkedList<'a, T>(pub &'a [T]);

/// Given the length in bytes of a packed list account `PkedList<T>`,
/// return the number of elems in it.
///
/// Returns `None` if given byte_len is not a valid length for
/// a PkedList of the given type `T`
///
/// Basically just `byte_len / size_of::<T>()`
#[inline]
pub const fn pked_list_len<T>(byte_len: usize) -> Option<usize> {
    // cant use a const here due to generic being outer
    let tlen: usize = size_of::<T>();
    if !byte_len.is_multiple_of(tlen) {
        return None;
    }
    Some(byte_len / tlen)
}

/// pointer casting "serde"
impl<'a, T> PkedList<'a, T> {
    #[inline]
    const fn of_acc_data_inner(acc_data: &'a [u8]) -> Option<Self> {
        let len = match pked_list_len::<T>(acc_data.len()) {
            None => return None,
            Some(x) => x,
        };
        Some(Self(unsafe {
            slice::from_raw_parts(acc_data.as_ptr().cast(), len)
        }))
    }

    /// Returns `None` if `acc_data` is not a valid list
    #[inline]
    pub const fn of_acc_data(acc_data: &'a [u8]) -> Option<Self> {
        const {
            assert!(align_of::<T>() == 1);
        }
        Self::of_acc_data_inner(acc_data)
    }

    /// # Safety
    /// - `acc_data` must have align >= align of `T`
    #[inline]
    pub const unsafe fn of_acc_data_unsafe(acc_data: &'a [u8]) -> Option<Self> {
        Self::of_acc_data_inner(acc_data)
    }

    #[inline]
    pub const fn as_acc_data(&self) -> &[u8] {
        let bytes = size_of_val(self.0);
        unsafe { slice::from_raw_parts(self.0.as_ptr().cast(), bytes) }
    }

    #[inline]
    pub const fn byte_offset(idx: usize) -> usize {
        idx * size_of::<T>()
    }

    #[inline]
    pub const fn byte_offset_checked(idx: usize) -> Option<usize> {
        idx.checked_mul(size_of::<T>())
    }
}

#[repr(transparent)]
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct PkedListMut<'a, T>(pub &'a mut [T]);

/// pointer casting "serde"
impl<'a, T> PkedListMut<'a, T> {
    #[inline]
    const fn of_acc_data_inner(acc_data: &'a mut [u8]) -> Option<Self> {
        let len = match pked_list_len::<T>(acc_data.len()) {
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
    /// - same requirements as [`PkedList::of_acc_data_unsafe`]
    #[inline]
    pub const unsafe fn of_acc_data_unsafe(acc_data: &'a mut [u8]) -> Option<Self> {
        Self::of_acc_data_inner(acc_data)
    }

    #[inline]
    pub const fn as_pked_list(&self) -> PkedList<'_, T> {
        PkedList(self.0)
    }
}
