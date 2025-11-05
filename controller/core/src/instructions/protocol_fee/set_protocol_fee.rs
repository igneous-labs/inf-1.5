use generic_array_struct::generic_array_struct;

use crate::instructions::internal_utils::{deser_borsh_opt_u16, ser_borsh_opt_u16};

// Accounts

#[generic_array_struct(builder pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct SetProtocolFeeIxAccs<T> {
    /// The pool's admin
    pub admin: T,

    /// The pool's state singleton PDA
    pub pool_state: T,
}

impl<T: Copy> SetProtocolFeeIxAccs<T> {
    #[inline]
    pub const fn memset(val: T) -> Self {
        Self([val; SET_PROTOCOL_FEE_IX_ACCS_LEN])
    }
}

pub type SetProtocolFeeIxKeys<'a> = SetProtocolFeeIxAccs<&'a [u8; 32]>;

pub type SetProtocolFeeIxKeysOwned = SetProtocolFeeIxAccs<[u8; 32]>;

pub type SetProtocolFeeIxAccFlags = SetProtocolFeeIxAccs<bool>;

pub const SET_PROTOCOL_FEE_IX_IS_WRITER: SetProtocolFeeIxAccFlags =
    SetProtocolFeeIxAccFlags::memset(false).const_with_pool_state(true);

pub const SET_PROTOCOL_FEE_IX_IS_SIGNER: SetProtocolFeeIxAccFlags =
    SetProtocolFeeIxAccFlags::memset(false).const_with_admin(true);

// Data

pub const SET_PROTOCOL_FEE_IX_MAX_DATA_LEN: usize = 7;
const _ASSERT_MAX_DATA_LEN: () =
    assert!(SET_PROTOCOL_FEE_IX_MAX_DATA_LEN == 1 + 2 * (1 + core::mem::size_of::<u16>()));

pub const SET_PROTOCOL_FEE_IX_DISCM: u8 = 11;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SetProtocolFeeIxArgs {
    pub trading_bps: Option<u16>,
    pub lp_bps: Option<u16>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SetProtocolFeeIxData {
    buf: [u8; SET_PROTOCOL_FEE_IX_MAX_DATA_LEN],
    len: usize,
}

impl SetProtocolFeeIxData {
    #[inline]
    pub const fn new(
        SetProtocolFeeIxArgs {
            trading_bps,
            lp_bps,
        }: SetProtocolFeeIxArgs,
    ) -> Self {
        let mut buf = [0u8; SET_PROTOCOL_FEE_IX_MAX_DATA_LEN];

        // Safety: all bounds below checked at compile-time,
        // buf should be big enough for everything

        let (discm, cursor) = unsafe { buf.split_first_mut().unwrap_unchecked() };
        *discm = SET_PROTOCOL_FEE_IX_DISCM;
        let cursor = unsafe { ser_borsh_opt_u16(cursor, trading_bps).unwrap_unchecked() };
        let cursor = unsafe { ser_borsh_opt_u16(cursor, lp_bps).unwrap_unchecked() };

        let len = SET_PROTOCOL_FEE_IX_MAX_DATA_LEN - cursor.len();

        Self { buf, len }
    }

    #[inline]
    pub const fn as_buf(&self) -> &[u8] {
        // using unsafe + ptr casting here to workaround
        // get_unchecked and indexing range not yet stable in const
        //
        // safety: self.len is in-bounds by construction
        unsafe { core::slice::from_raw_parts(self.buf.as_ptr(), self.len) }
    }

    /// Returns `None` if data is invalid
    #[inline]
    pub const fn parse_no_discm(data: &[u8]) -> Option<SetProtocolFeeIxArgs> {
        let (trading_bps, data) = match deser_borsh_opt_u16(data) {
            None => return None,
            Some(x) => x,
        };
        let lp_bps = match deser_borsh_opt_u16(data) {
            Some((x, &[])) => x,
            // also err if data is too long
            _invalid => return None,
        };
        Some(SetProtocolFeeIxArgs {
            trading_bps,
            lp_bps,
        })
    }
}

#[cfg(test)]
mod tests {
    use borsh::{BorshDeserialize, BorshSerialize};
    use proptest::{option, prelude::*};

    use super::*;

    #[derive(BorshDeserialize, BorshSerialize)]
    struct BorshSerde {
        pub trading_bps: Option<u16>,
        pub lp_bps: Option<u16>,
    }

    proptest! {
        #[test]
        fn check_serde_against_borsh(
            trading_bps in option::of(any::<u16>()),
            lp_bps in option::of(any::<u16>()),
        ) {
            let us = SetProtocolFeeIxData::new(
                SetProtocolFeeIxArgs { trading_bps, lp_bps }
            );
            let us_data = &us.as_buf()[1..];
            let b = BorshSerde { trading_bps, lp_bps };
            let b_data = &borsh::to_vec(&b).unwrap();
            prop_assert_eq!(us_data, b_data);

            // serialize roundtrip using each other's data
            let us_rt = SetProtocolFeeIxData::parse_no_discm(b_data).unwrap();
            let mut us_data_mut = us_data;
            let b_rt = BorshSerde::deserialize(&mut us_data_mut).unwrap();

            prop_assert_eq!(us_rt.trading_bps, b_rt.trading_bps);
            prop_assert_eq!(us_rt.lp_bps, b_rt.lp_bps);
        }
    }
}
