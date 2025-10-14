use core::{
    mem::{align_of, size_of},
    slice,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PackedList<'a, T>(pub &'a [T]);

/// pointer casting "serde"
impl<'a, T> PackedList<'a, T> {
    #[inline]
    pub const fn of_acc_data(acc_data: &'a [u8]) -> Option<Self> {
        const {
            assert!(align_of::<T>() == 1);
        }

        let tlen: usize = size_of::<T>();
        // is_multiple_of doesnt exist in rustc 1.84
        #[allow(clippy::manual_is_multiple_of)]
        if acc_data.len() % tlen != 0 {
            return None;
        }
        let len = acc_data.len() / tlen;
        Some(Self(unsafe {
            slice::from_raw_parts(acc_data.as_ptr().cast(), len)
        }))
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
    pub const fn of_acc_data(acc_data: &'a mut [u8]) -> Option<Self> {
        const {
            assert!(align_of::<T>() == 1);
        }

        let tlen: usize = size_of::<T>();
        // is_multiple_of doesnt exist in rustc 1.84
        #[allow(clippy::manual_is_multiple_of)]
        if acc_data.len() % tlen != 0 {
            return None;
        }
        let len = acc_data.len() / tlen;
        Some(Self(unsafe {
            slice::from_raw_parts_mut(acc_data.as_mut_ptr().cast(), len)
        }))
    }

    #[inline]
    pub const fn as_packed_list(&self) -> PackedList<'_, T> {
        PackedList(self.0)
    }
}
