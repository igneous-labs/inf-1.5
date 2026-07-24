use generic_array_struct::generic_array_struct;

/// Common prefix for all admin instructions
#[generic_array_struct(all pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct AdminIxPreAccs<T> {
    /// The `PricingState` PDA
    pub pricing_state: T,

    /// Current `pricing_state.admin`
    pub admin: T,
}

impl<T: Copy> AdminIxPreAccs<T> {
    #[inline]
    pub const fn memset(val: T) -> Self {
        Self([val; ADMIN_IX_PRE_ACCS_LEN])
    }
}

impl<T> AdminIxPreAccs<T> {
    #[inline]
    pub const fn new(arr: [T; ADMIN_IX_PRE_ACCS_LEN]) -> Self {
        Self(arr)
    }
}

pub type AdminIxPreKeys<'a> = AdminIxPreAccs<&'a [u8; 32]>;
pub type AdminIxPreKeysOwned = AdminIxPreAccs<[u8; 32]>;
pub type AdminIxPreAccFlags = AdminIxPreAccs<bool>;

/// `pricing_state` writable since the admin is changing something on it
pub const ADMIN_IX_PRE_IS_WRITER: AdminIxPreAccFlags =
    AdminIxPreAccFlags::const_from_destr(AdminIxPreAccsDestr {
        pricing_state: true,
        admin: false,
    });

/// Signed by `admin`
pub const ADMIN_IX_PRE_IS_SIGNER: AdminIxPreAccFlags =
    AdminIxPreAccFlags::const_from_destr(AdminIxPreAccsDestr {
        pricing_state: false,
        admin: true,
    });
