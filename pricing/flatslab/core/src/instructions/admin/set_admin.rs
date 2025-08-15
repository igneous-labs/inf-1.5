use generic_array_struct::generic_array_struct;

// Accounts

#[generic_array_struct(builder pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct SetAdminIxAccs<T> {
    /// The current program admin
    pub current_admin: T,

    /// The new program admin to set to
    pub new_admin: T,

    /// The slab PDA
    pub slab: T,
}

impl<T: Copy> SetAdminIxAccs<T> {
    #[inline]
    pub const fn memset(v: T) -> Self {
        Self([v; SET_ADMIN_IX_ACCS_LEN])
    }

    /// For more convenient usage with type aliases
    #[inline]
    pub const fn new(arr: [T; SET_ADMIN_IX_ACCS_LEN]) -> Self {
        Self(arr)
    }
}

pub type SetAdminIxKeys<'a> = SetAdminIxAccs<&'a [u8; 32]>;

pub type SetAdminIxKeysOwned = SetAdminIxAccs<[u8; 32]>;

pub type SetAdminIxAccFlags = SetAdminIxAccs<bool>;

pub const SET_ADMIN_IX_IS_WRITER: SetAdminIxAccFlags =
    SetAdminIxAccFlags::memset(false).const_with_slab(true);

pub const SET_ADMIN_IX_IS_SIGNER: SetAdminIxAccFlags =
    SetAdminIxAccFlags::memset(false).const_with_current_admin(true);

// Data

pub const SET_ADMIN_IX_DISCM: u8 = 254;

pub const SET_ADMIN_IX_DATA_LEN: usize = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SetAdminIxData([u8; SET_ADMIN_IX_DATA_LEN]);

impl SetAdminIxData {
    #[inline]
    pub const fn new() -> Self {
        Self([SET_ADMIN_IX_DISCM])
    }

    #[inline]
    pub const fn as_buf(&self) -> &[u8; SET_ADMIN_IX_DATA_LEN] {
        &self.0
    }
}

impl Default for SetAdminIxData {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
