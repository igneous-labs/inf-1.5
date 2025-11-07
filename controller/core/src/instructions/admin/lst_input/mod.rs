use generic_array_struct::generic_array_struct;

pub mod disable;
pub mod enable;

// Accounts

#[generic_array_struct(builder pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct SetLstInputIxAccs<T> {
    /// The pool's admin
    pub admin: T,

    /// Mint of the LST to en/disable input of
    pub lst_mint: T,

    /// The pool's state singleton PDA
    pub pool_state: T,

    /// Dynamic list PDA of LstStates for each LST in the pool
    pub lst_state_list: T,
}

impl<T> SetLstInputIxAccs<T> {
    /// For ease of use with type aliases
    #[inline]
    pub const fn new(a: [T; SET_LST_INPUT_IX_ACCS_LEN]) -> Self {
        Self(a)
    }
}

impl<T: Copy> SetLstInputIxAccs<T> {
    #[inline]
    pub const fn memset(val: T) -> Self {
        Self([val; SET_LST_INPUT_IX_ACCS_LEN])
    }
}

pub type SetLstInputIxKeys<'a> = SetLstInputIxAccs<&'a [u8; 32]>;

pub type SetLstInputIxKeysOwned = SetLstInputIxAccs<[u8; 32]>;

pub type SetLstInputIxAccFlags = SetLstInputIxAccs<bool>;

impl<T> AsRef<[T]> for SetLstInputIxAccs<T> {
    #[inline]
    fn as_ref(&self) -> &[T] {
        &self.0
    }
}

pub const SET_LST_INPUT_IX_IS_WRITER: SetLstInputIxAccFlags =
    SetLstInputIxAccFlags::memset(false).const_with_lst_state_list(true);

pub const SET_LST_INPUT_IX_IS_SIGNER: SetLstInputIxAccFlags =
    SetLstInputIxAccFlags::memset(false).const_with_admin(true);
