use generic_array_struct::generic_array_struct;

use crate::instructions::internal_utils::caba;

// Accounts

#[generic_array_struct(builder pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct SetRpsIxAccs<T> {
    /// The pool's state singleton PDA
    pub pool_state: T,

    /// The pool's RPS authority
    pub rps_auth: T,
}

impl<T: Copy> SetRpsIxAccs<T> {
    #[inline]
    pub const fn memset(val: T) -> Self {
        Self([val; SET_RPS_IX_ACCS_LEN])
    }
}

pub type SetRpsIxKeys<'a> = SetRpsIxAccs<&'a [u8; 32]>;

pub type SetRpsIxKeysOwned = SetRpsIxAccs<[u8; 32]>;

pub type SetRpsIxAccFlags = SetRpsIxAccs<bool>;

pub const SET_RPS_IX_IS_WRITER: SetRpsIxAccFlags =
    SetRpsIxAccFlags::memset(false).const_with_pool_state(true);

pub const SET_RPS_IX_IS_SIGNER: SetRpsIxAccFlags =
    SetRpsIxAccFlags::memset(false).const_with_rps_auth(true);

// Data

pub const SET_RPS_IX_DISCM: u8 = 26;

pub const SET_RPS_IX_DATA_LEN: usize = 9;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct SetRpsIxData([u8; SET_RPS_IX_DATA_LEN]);

impl SetRpsIxData {
    #[inline]
    pub const fn new(rps: u64) -> Self {
        const A: usize = SET_RPS_IX_DATA_LEN;

        let mut d = [0u8; A];

        d = caba::<A, 0, 1>(d, &[SET_RPS_IX_DISCM]);
        d = caba::<A, 1, 8>(d, &rps.to_le_bytes());

        Self(d)
    }

    #[inline]
    pub const fn as_buf(&self) -> &[u8; SET_RPS_IX_DATA_LEN] {
        &self.0
    }

    /// Returns `rps` arg, the new RPS to set to
    #[inline]
    pub const fn parse_no_discm(data: &[u8; 8]) -> u64 {
        u64::from_le_bytes(*data)
    }
}
