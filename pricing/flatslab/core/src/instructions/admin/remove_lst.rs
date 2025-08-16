use generic_array_struct::generic_array_struct;

// Accounts

#[generic_array_struct(builder pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct RemoveLstIxAccs<T> {
    /// The program admin
    pub admin: T,

    /// Account to refund SOL rent to
    pub refund_rent_to: T,

    /// The slab PDA
    pub slab: T,

    /// Mint of the LST to set fees for
    pub mint: T,
}

impl<T: Copy> RemoveLstIxAccs<T> {
    #[inline]
    pub const fn memset(v: T) -> Self {
        Self([v; REMOVE_LST_IX_ACCS_LEN])
    }
}

pub type RemoveLstIxKeys<'a> = RemoveLstIxAccs<&'a [u8; 32]>;

pub type RemoveLstIxKeysOwned = RemoveLstIxAccs<[u8; 32]>;

pub type RemoveLstIxAccFlags = RemoveLstIxAccs<bool>;

pub const REMOVE_LST_IX_IS_WRITER: RemoveLstIxAccFlags = RemoveLstIxAccFlags::memset(false)
    .const_with_refund_rent_to(true)
    .const_with_slab(true);

pub const REMOVE_LST_IX_IS_SIGNER: RemoveLstIxAccFlags =
    RemoveLstIxAccFlags::memset(false).const_with_admin(true);

// Data

pub const REMOVE_LST_IX_DISCM: u8 = 252;

pub const REMOVE_LST_IX_DATA_LEN: usize = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RemoveLstIxData([u8; REMOVE_LST_IX_DATA_LEN]);

impl RemoveLstIxData {
    #[inline]
    pub const fn new() -> Self {
        Self([REMOVE_LST_IX_DISCM])
    }

    #[inline]
    pub const fn as_buf(&self) -> &[u8; REMOVE_LST_IX_DATA_LEN] {
        &self.0
    }
}

impl Default for RemoveLstIxData {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
