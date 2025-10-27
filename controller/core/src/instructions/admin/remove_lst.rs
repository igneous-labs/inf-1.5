use generic_array_struct::generic_array_struct;

use crate::instructions::internal_utils::caba;

// Accounts

#[generic_array_struct(builder pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct RemoveLstIxAccs<T> {
    /// The pool's admin
    pub admin: T,

    /// Account to refund SOL rent to
    pub refund_rent_to: T,

    /// Mint of the LST to remove
    pub lst_mint: T,

    /// LST reserves token account to destroy
    pub pool_reserves: T,

    /// The LST protocol fee accumulator token account to destroy
    pub protocol_fee_accumulator: T,

    /// The protocol fee accumulator token account authority PDA. PDA ["protocol_fee"]
    pub protocol_fee_accumulator_auth: T,

    /// The pool's state singleton PDA
    pub pool_state: T,

    /// Dynamic list PDA of LstStates for each LST in the pool
    pub lst_state_list: T,

    /// Token program of the LST to remove
    pub lst_token_program: T,
}

impl<T: Copy> RemoveLstIxAccs<T> {
    #[inline]
    pub const fn memset(val: T) -> Self {
        Self([val; REMOVE_LST_IX_ACCS_LEN])
    }
}

pub type RemoveLstIxKeys<'a> = RemoveLstIxAccs<&'a [u8; 32]>;

pub type RemoveLstIxKeysOwned = RemoveLstIxAccs<[u8; 32]>;

pub type RemoveLstIxAccFlags = RemoveLstIxAccs<bool>;

impl<T> AsRef<[T]> for RemoveLstIxAccs<T> {
    #[inline]
    fn as_ref(&self) -> &[T] {
        &self.0
    }
}

pub const REMOVE_LST_IX_IS_WRITER: RemoveLstIxAccFlags = RemoveLstIxAccFlags::memset(true)
    .const_with_refund_rent_to(true)
    .const_with_pool_reserves(true)
    .const_with_protocol_fee_accumulator(true)
    .const_with_protocol_fee_accumulator_auth(true)
    .const_with_lst_state_list(true);

pub const REMOVE_LST_IX_IS_SIGNER: RemoveLstIxAccFlags =
    RemoveLstIxAccFlags::memset(false).const_with_admin(true);

// Data

pub const REMOVE_LST_IX_DISCM: u8 = 8;

pub const REMOVE_LST_IX_DATA_LEN: usize = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct RemoveLstIxData([u8; REMOVE_LST_IX_DATA_LEN]);

impl RemoveLstIxData {
    #[inline]
    pub const fn new(lst_idx: u32) -> Self {
        const A: usize = REMOVE_LST_IX_DATA_LEN;

        let mut d = [0u8; A];

        d = caba::<A, 0, 1>(d, &[REMOVE_LST_IX_DISCM]);
        d = caba::<A, 1, 4>(d, &lst_idx.to_le_bytes());

        Self(d)
    }

    #[inline]
    pub const fn as_buf(&self) -> &[u8; REMOVE_LST_IX_DATA_LEN] {
        &self.0
    }

    /// Returns `lst_idx`
    #[inline]
    pub const fn parse_no_discm(data: &[u8; 4]) -> u32 {
        u32::from_le_bytes(*data)
    }
}
