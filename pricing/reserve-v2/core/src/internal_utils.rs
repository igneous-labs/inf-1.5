// TODO: add `$vis:vis` arg for controlling visibility of methods
/// Implement pointer casting "deserialization" for an account struct.
/// Only available in targets that have same endianness as solana VM (little-endian)
///
/// # Safety
/// This should only be used for types that are:
/// - `repr(C)`
/// - `core::mem::size_of::<Self> == account data length`,
/// - have no internal struct padding. External struct padding is ok.
///
/// # Args
/// - Include `packed` arg if struct is a packed byte array
///   (`core::mem::align_of::<Self> == 1`, endianness does not matter)
macro_rules! impl_cast_from_acc_data {
    // not packed
    ($Ty:ty) => {
        #[cfg(target_endian = "little")]
        impl $Ty {
            /// # Safety
            /// - `acc_data_arr` must have the same align as Self.
            #[inline]
            pub const unsafe fn of_acc_data_arr(
                acc_data_arr: &[u8; core::mem::size_of::<Self>()],
            ) -> &Self {
                // safety: Self has no internal struct padding
                &*core::ptr::from_ref(acc_data_arr).cast()
            }

            /// # Safety
            /// - `acc_data_arr` must have the same align as Self.
            #[inline]
            pub const unsafe fn of_acc_data(
                acc_data: &[u8],
            ) -> Option<&Self> {
                const LEN: usize = core::mem::size_of::<$Ty>();

                match acc_data.len() {
                    // safety:
                    // - Self has no internal struct padding
                    // - align safety precondition
                    // - length == LEN checked
                    LEN => Some(Self::of_acc_data_unchecked(acc_data)),
                    _ => None,
                }
            }

            impl_cast_from_acc_data!(@internal);
        }
    };

    // Packed
    ($Ty:ty, packed) => {
        impl $Ty {
            #[inline]
            pub const fn of_acc_data_arr(
                acc_data_arr: &[u8; core::mem::size_of::<Self>()],
            ) -> &Self {
                const {
                    assert!(core::mem::align_of::<Self>() == 1);
                }

                // safety:
                // - Self has no internal struct padding
                // - align == 1 checked at compile-time above
                unsafe { &*core::ptr::from_ref(acc_data_arr).cast() }
            }

            #[inline]
            pub const fn of_acc_data(
                acc_data: &[u8],
            ) -> Option<&Self> {
                const LEN: usize = core::mem::size_of::<$Ty>();

                match acc_data.len() {
                    // safety:
                    // - Self has no internal struct padding
                    // - align == 1 checked at compile-time above
                    // - length == LEN checked
                    LEN => Some(unsafe { Self::of_acc_data_unchecked(acc_data) }),
                    _ => None,
                }
            }

            impl_cast_from_acc_data!(@internal);
        }
    };

    // rest of the impl thats common between packed and no packed arg
    (@internal) => {
        /// # Safety
        /// - `acc_data` must be of `size_of::<Self>()`
        /// - `acc_data` must have the same align as Self
        #[inline]
        pub const unsafe fn of_acc_data_unchecked(acc_data: &[u8]) -> &Self {
            Self::of_acc_data_arr(&*acc_data.as_ptr().cast())
        }
    };
}
pub(crate) use impl_cast_from_acc_data;

/// Implement pointer casting "serialization" for an account struct.
/// Only available in targets that have same endianness as solana VM (little-endian)
///
/// # Safety
/// This should only be used for types that are:
/// - `repr(C)`
/// - `core::mem::size_of::<Self> == account data length`,
/// - have no internal struct padding. External struct padding is ok.
///
/// # Args
/// - Include `packed` arg if struct is a packed byte array
///   (`core::mem::align_of::<Self> == 1`, endianness does not matter)
macro_rules! impl_cast_to_acc_data {
    ($Ty:ty) => {
        #[cfg(target_endian = "little")]
        impl_cast_to_acc_data!(@internal $Ty);
    };

    ($Ty:ty, packed) => {
        impl_cast_to_acc_data!(@internal $Ty);
    };

    // rest of the impl thats common between packed and no packed arg
    (@internal $Ty:ty) => {
        impl $Ty {
            #[inline]
            pub const fn as_acc_data_arr(&self) -> &[u8; core::mem::size_of::<Self>()] {
                // safety:
                // - Self has no internal padding. Presence of external/suffix
                //   padding just means those bytes are not included in the returned array ref.
                unsafe { &*core::ptr::from_ref(self).cast() }
            }
        }
    };
}
pub(crate) use impl_cast_to_acc_data;

macro_rules! const_map {
    ($DEFAULT:expr, $from_arr:expr, $const_fn:expr) => {{
        let mut res = [$DEFAULT; _];
        let mut i = 0;
        while i < res.len() {
            res[i] = $const_fn(&$from_arr[i]);
            i += 1;
        }
        res
    }};
}
pub(crate) use const_map;

/// caba = `const_assign_byte_array`
pub(crate) const fn caba<const A: usize, const START: usize, const LEN: usize>(
    mut arr: [u8; A],
    val: &[u8; LEN],
) -> [u8; A] {
    const {
        assert!(START + LEN <= A);
    }

    let mut i = 0;
    while i < LEN {
        arr[START + i] = val[i];
        i += 1;
    }
    arr
}

/// csba = `const_split_byte_array`
#[inline]
pub(crate) const fn csba<const M: usize, const N: usize, const X: usize>(
    data: &[u8; M],
) -> (&[u8; N], &[u8; X]) {
    const {
        assert!(N <= M);
        assert!(X == M - N)
    }

    // Safety: bounds checked above
    let (a, b) = unsafe { data.split_at_unchecked(N) };

    // SAFETY: data is guaranteed to be of length M
    // and we are splitting it into two slices of length N and X (i.e M-N)
    (unsafe { &*a.as_ptr().cast::<[u8; N]>() }, unsafe {
        &*b.as_ptr().cast::<[u8; X]>()
    })
}
