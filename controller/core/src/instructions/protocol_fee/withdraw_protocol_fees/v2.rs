use generic_array_struct::generic_array_struct;

use crate::instructions::generic::DiscmOnlyIxData;

// Accounts

#[generic_array_struct(builder pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct WithdrawProtocolFeesV2IxAccs<T> {
    /// The pool's state singleton PDA
    pub pool_state: T,

    /// The pool's protocol fee beneficiary
    pub beneficiary: T,

    /// INF token account to withdraw unclaimed protocol fees to
    pub withdraw_to: T,

    /// INF token mint
    pub inf_mint: T,

    /// INF token program
    pub token_program: T,
}

impl<T: Copy> WithdrawProtocolFeesV2IxAccs<T> {
    #[inline]
    pub const fn memset(val: T) -> Self {
        Self([val; WITHDRAW_PROTOCOL_FEES_V2_IX_ACCS_LEN])
    }
}

pub type WithdrawProtocolFeesV2IxKeys<'a> = WithdrawProtocolFeesV2IxAccs<&'a [u8; 32]>;

pub type WithdrawProtocolFeesV2IxKeysOwned = WithdrawProtocolFeesV2IxAccs<[u8; 32]>;

pub type WithdrawProtocolFeesV2IxAccFlags = WithdrawProtocolFeesV2IxAccs<bool>;

pub const WITHDRAW_PROTOCOL_FEES_V2_IX_IS_WRITER: WithdrawProtocolFeesV2IxAccFlags =
    WithdrawProtocolFeesV2IxAccFlags::memset(false)
        .const_with_pool_state(true)
        .const_with_withdraw_to(true)
        .const_with_inf_mint(true);

pub const WITHDRAW_PROTOCOL_FEES_V2_IX_IS_SIGNER: WithdrawProtocolFeesV2IxAccFlags =
    WithdrawProtocolFeesV2IxAccFlags::memset(false).const_with_beneficiary(true);

// Data

pub const WITHDRAW_PROTOCOL_FEES_V2_IX_DISCM: u8 = 25;

pub type WithdrawProtocolFeesV2IxData = DiscmOnlyIxData<WITHDRAW_PROTOCOL_FEES_V2_IX_DISCM>;

pub const WITHDRAW_PROTOCOL_FEES_V2_IX_DATA_LEN: usize = WithdrawProtocolFeesV2IxData::DATA_LEN;
