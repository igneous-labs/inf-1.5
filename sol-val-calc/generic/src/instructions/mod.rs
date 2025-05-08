use generic_array_struct::generic_array_struct;

pub mod lst_to_sol;

pub mod sol_to_lst;

#[generic_array_struct(builder pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct IxSufAccs<T> {
    pub state: T,
    pub pool_state: T,
    pub pool_prog: T,
    pub pool_progdata: T,
}

impl<T: Copy> IxSufAccs<T> {
    #[inline]
    pub const fn memset(val: T) -> Self {
        Self([val; IX_SUF_ACCS_LEN])
    }
}

impl<T> AsRef<[T]> for IxSufAccs<T> {
    #[inline]
    fn as_ref(&self) -> &[T] {
        &self.0
    }
}

pub type IxSufKeys<'a> = IxSufAccs<&'a [u8; 32]>;

pub type IxSufKeysOwned = IxSufAccs<[u8; 32]>;

pub type IxSufAccFlags = IxSufAccs<bool>;

pub const IX_SUF_IS_WRITER: IxSufAccFlags = IxSufAccFlags::memset(false);

pub const IX_SUF_IS_SIGNER: IxSufAccFlags = IxSufAccFlags::memset(false);

impl IxSufKeys<'_> {
    #[inline]
    pub fn into_owned(&self) -> IxSufKeysOwned {
        IxSufAccs(self.0.map(|p| *p))
    }
}

impl IxSufKeysOwned {
    #[inline]
    pub fn as_borrowed(&self) -> IxSufKeys<'_> {
        IxSufAccs(self.0.each_ref())
    }
}
