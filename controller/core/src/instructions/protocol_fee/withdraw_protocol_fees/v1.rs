use generic_array_struct::generic_array_struct;

use crate::instructions::internal_utils::caba;

// Accounts

#[generic_array_struct(builder pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct WithdrawProtocolFeesIxAccs<T> {
    /// The pool's protocol fee beneficiary
    pub beneficiary: T,

    /// Token account to withdraw accumulated protocol fees to
    pub withdraw_to: T,

    /// LST protocol fee accmulator token account
    pub protocol_fee_accumulator: T,

    /// The protocol fee accumulator token account authority PDA. PDA ["protocol_fee"]
    pub protocol_fee_accumulator_auth: T,

    /// Token program of the LST
    pub token_program: T,

    /// The pool's state singleton PDA
    pub pool_state: T,

    pub lst_mint: T,
}

impl<T: Copy> WithdrawProtocolFeesIxAccs<T> {
    #[inline]
    pub const fn memset(val: T) -> Self {
        Self([val; WITHDRAW_PROTOCOL_FEES_IX_ACCS_LEN])
    }
}

pub type WithdrawProtocolFeesIxKeys<'a> = WithdrawProtocolFeesIxAccs<&'a [u8; 32]>;

pub type WithdrawProtocolFeesIxKeysOwned = WithdrawProtocolFeesIxAccs<[u8; 32]>;

pub type WithdrawProtocolFeesIxAccFlags = WithdrawProtocolFeesIxAccs<bool>;

pub const WITHDRAW_PROTOCOL_FEES_IX_IS_WRITER: WithdrawProtocolFeesIxAccFlags =
    WithdrawProtocolFeesIxAccFlags::memset(false)
        .const_with_withdraw_to(true)
        .const_with_protocol_fee_accumulator(true);

pub const WITHDRAW_PROTOCOL_FEES_IX_IS_SIGNER: WithdrawProtocolFeesIxAccFlags =
    WithdrawProtocolFeesIxAccFlags::memset(false).const_with_beneficiary(true);

// Data

pub const WITHDRAW_PROTOCOL_FEES_IX_DISCM: u8 = 14;

pub const WITHDRAW_PROTOCOL_FEES_IX_DATA_LEN: usize = 9;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct WithdrawProtocolFeesIxData([u8; WITHDRAW_PROTOCOL_FEES_IX_DATA_LEN]);

impl WithdrawProtocolFeesIxData {
    #[inline]
    pub const fn new(amt: u64) -> Self {
        const A: usize = WITHDRAW_PROTOCOL_FEES_IX_DATA_LEN;

        let mut d = [0u8; A];

        d = caba::<A, 0, 1>(d, &[WITHDRAW_PROTOCOL_FEES_IX_DISCM]);
        d = caba::<A, 1, 8>(d, &amt.to_le_bytes());

        Self(d)
    }

    #[inline]
    pub const fn as_buf(&self) -> &[u8; WITHDRAW_PROTOCOL_FEES_IX_DATA_LEN] {
        &self.0
    }

    /// Returns `amt` arg, the amount of LST tokens to withdraw
    #[inline]
    pub const fn parse_no_discm(data: &[u8; 8]) -> u64 {
        u64::from_le_bytes(*data)
    }
}
