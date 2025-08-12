use generic_array_struct::generic_array_struct;

// Accounts

#[generic_array_struct(builder pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct InitIxAccs<T> {
    /// The signer paying for the slab account's rent
    pub payer: T,

    /// The slab PDA to initialize
    pub slab: T,

    /// System program
    pub system_program: T,
}

impl<T: Copy> InitIxAccs<T> {
    #[inline]
    pub const fn memset(v: T) -> Self {
        Self([v; INIT_IX_ACCS_LEN])
    }
}

pub type InitIxKeys<'a> = InitIxAccs<&'a [u8; 32]>;

pub type InitIxKeysOwned = InitIxAccs<[u8; 32]>;

pub type InitIxAccFlags = InitIxAccs<bool>;

pub const INIT_IX_IS_WRITER: InitIxAccFlags =
    InitIxAccFlags::memset(true).const_with_system_program(false);

pub const INIT_IX_IS_SIGNER: InitIxAccFlags = InitIxAccFlags::memset(false).const_with_payer(true);

// Data

pub const INIT_IX_DISCM: u8 = 255;

pub const INIT_IX_DATA_LEN: usize = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InitIxData([u8; INIT_IX_DATA_LEN]);

impl InitIxData {
    #[inline]
    pub const fn new() -> Self {
        Self([INIT_IX_DISCM])
    }

    #[inline]
    pub const fn as_buf(&self) -> &[u8; INIT_IX_DATA_LEN] {
        &self.0
    }
}

impl Default for InitIxData {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
