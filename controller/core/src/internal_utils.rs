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

            /// # Returns
            /// `None` if `acc_data` does not have the same length as Self
            ///
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

            /// # Returns
            /// Same as [`Self::of_acc_data_arr`]
            ///
            /// # Safety
            /// Same as [`Self::of_acc_data_arr`]
            #[inline]
            pub const unsafe fn of_acc_data_arr_mut(
                acc_data_arr: &mut [u8; core::mem::size_of::<Self>()],
            ) -> &mut Self {
                // safety: Self has no internal struct padding
                &mut *core::ptr::from_mut(acc_data_arr).cast()
            }

            /// # Returns
            /// Same as [`Self::of_acc_data`]
            ///
            /// # Safety
            /// Same as [`Self::of_acc_data`]
            #[inline]
            pub const unsafe fn of_acc_data_mut(
                acc_data: &mut [u8],
            ) -> Option<&mut Self> {
                const LEN: usize = core::mem::size_of::<$Ty>();

                match acc_data.len() {
                    // safety:
                    // - Self has no internal struct padding
                    // - align safety precondition
                    // - length == LEN checked
                    LEN => Some(Self::of_acc_data_unchecked_mut(acc_data)),
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

            /// # Returns
            /// `None` if `acc_data` does not have the same length as Self
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

            #[inline]
            pub const fn of_acc_data_arr_mut(
                acc_data_arr: &mut [u8; core::mem::size_of::<Self>()],
            ) -> &mut Self {
                // safety:
                // - Self has no internal struct padding
                // - align == 1 checked at compile-time above
                unsafe { &mut *core::ptr::from_mut(acc_data_arr).cast() }
            }

            /// # Returns
            /// Same as [`Self::of_acc_data`]
            #[inline]
            pub const fn of_acc_data_mut(
                acc_data: &mut [u8],
            ) -> Option<&mut Self> {
                const LEN: usize = core::mem::size_of::<$Ty>();

                match acc_data.len() {
                    // safety:
                    // - Self has no internal struct padding
                    // - align == 1 checked at compile-time above
                    // - length == LEN checked
                    LEN => Some(unsafe { Self::of_acc_data_unchecked_mut(acc_data) }),
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

        /// # Safety
        /// - `acc_data` must be of `size_of::<Self>()`
        /// - `acc_data` must have the same align as Self
        #[inline]
        pub const unsafe fn of_acc_data_unchecked_mut(acc_data: &mut [u8]) -> &mut Self {
            Self::of_acc_data_arr_mut(&mut *acc_data.as_mut_ptr().cast())
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

macro_rules! impl_verify_vers {
    ($Ty:ty, $EXPECTED:expr) => {
        impl $Ty {
            #[inline]
            pub const fn verify_vers(&self) -> Result<(), $crate::err::WrongVersErr> {
                if self.version == $EXPECTED {
                    Ok(())
                } else {
                    Err($crate::err::WrongVersErr {
                        actual: self.version,
                        expected: $EXPECTED,
                    })
                }
            }
        }
    };
}
pub(crate) use impl_verify_vers;

macro_rules! impl_gas_memset {
    (
        $GasStruct:ident,
        // TODO: looks like Self::LEN is still useful to export
        $LEN:expr
    ) => {
        impl<T: Copy> $GasStruct<T> {
            #[inline]
            pub const fn memset(v: T) -> Self {
                Self([v; $LEN])
            }
        }
    };
}
pub(crate) use impl_gas_memset;
