use core::{
    mem::{align_of, size_of},
    slice,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PackedList<'a, T>(&'a [T]);

/// pointer casting "serde"
impl<T> PackedList<'_, T> {
    #[inline]
    pub const fn of_acc_data(acc_data: &[u8]) -> Option<Self> {
        const {
            assert!(align_of::<T>() == 1);
        }

        let tlen: usize = size_of::<T>();
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
        let bytes = self.0.len() * size_of::<T>();
        unsafe { slice::from_raw_parts(self.0.as_ptr().cast(), bytes) }
    }
}
