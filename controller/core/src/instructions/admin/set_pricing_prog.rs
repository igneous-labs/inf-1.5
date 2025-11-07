use generic_array_struct::generic_array_struct;

use crate::instructions::generic::DiscmOnlyIxData;

// Accounts

#[generic_array_struct(builder pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct SetPricingProgIxAccs<T> {
    /// The pool's current admin
    pub admin: T,

    /// New pricing program to set to
    pub new: T,

    /// The pool's state singleton PDA
    pub pool_state: T,
}

impl<T: Copy> SetPricingProgIxAccs<T> {
    #[inline]
    pub const fn memset(val: T) -> Self {
        Self([val; SET_PRICING_PROG_IX_ACCS_LEN])
    }
}

pub type SetPricingProgIxKeys<'a> = SetPricingProgIxAccs<&'a [u8; 32]>;

pub type SetPricingProgIxKeysOwned = SetPricingProgIxAccs<[u8; 32]>;

pub type SetPricingProgIxAccFlags = SetPricingProgIxAccs<bool>;

pub const SET_PRICING_PROG_IX_IS_WRITER: SetPricingProgIxAccFlags =
    SetPricingProgIxAccFlags::memset(false).const_with_pool_state(true);

pub const SET_PRICING_PROG_IX_IS_SIGNER: SetPricingProgIxAccFlags =
    SetPricingProgIxAccFlags::memset(false).const_with_admin(true);

// Data

pub const SET_PRICING_PROG_IX_DISCM: u8 = 13;

pub type SetPricingProgIxData = DiscmOnlyIxData<SET_PRICING_PROG_IX_DISCM>;

pub const SET_PRICING_PROG_IX_DATA_LEN: usize = SetPricingProgIxData::DATA_LEN;
