use core::{iter::Chain, slice};

use generic_array_struct::generic_array_struct;

pub mod exact_in;
pub mod exact_out;

// Accounts

#[generic_array_struct(builder pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct IxPreAccs<T> {
    pub input_mint: T,
    pub output_mint: T,
}

impl<T> IxPreAccs<T> {
    /// For more convenient usage with type aliases
    #[inline]
    pub const fn new(arr: [T; IX_PRE_ACCS_LEN]) -> Self {
        Self(arr)
    }
}

impl<T: Copy> IxPreAccs<T> {
    #[inline]
    pub const fn memset(val: T) -> Self {
        Self([val; IX_PRE_ACCS_LEN])
    }
}

pub type IxPreKeys<'a> = IxPreAccs<&'a [u8; 32]>;

pub type IxPreKeysOwned = IxPreAccs<[u8; 32]>;

pub type IxPreAccFlags = IxPreAccs<bool>;

pub const IX_PRE_IS_WRITER: IxPreAccFlags = IxPreAccFlags::memset(false);

pub const IX_PRE_IS_SIGNER: IxPreAccFlags = IxPreAccFlags::memset(false);

impl IxPreKeys<'_> {
    #[inline]
    pub fn into_owned(&self) -> IxPreKeysOwned {
        IxPreAccs(self.0.map(|p| *p))
    }
}

impl IxPreKeysOwned {
    #[inline]
    pub fn as_borrowed(&self) -> IxPreKeys<'_> {
        IxPreAccs(self.0.each_ref())
    }
}

// Genericized Input

pub struct IxAccs<T, P> {
    /// Interface account prefix; [`IxPreAccs`]
    pub ix_prefix: IxPreAccs<T>,

    /// Account suffix specific to each implementation
    pub suf: P,
}

impl<T, P> IxAccs<T, P> {
    /// For more convenient usage with type aliases
    #[inline]
    pub const fn new(ix_prefix: IxPreAccs<T>, suf: P) -> Self {
        Self { ix_prefix, suf }
    }
}

pub type AccsIter<'a, T> = Chain<slice::Iter<'a, T>, slice::Iter<'a, T>>;

impl<T, P: AsRef<[T]>> IxAccs<T, P> {
    #[inline]
    pub fn seq(&self) -> AccsIter<'_, T> {
        let Self { ix_prefix, suf } = self;
        ix_prefix.0.iter().chain(suf.as_ref().iter())
    }
}
