use generic_array_struct::generic_array_struct;

pub mod lst_to_sol;
pub mod sol_to_lst;

mod internal_utils;

#[generic_array_struct(builder pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct IxPreAccs<T> {
    pub lst_mint: T,
}

impl<T> IxPreAccs<T> {
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
