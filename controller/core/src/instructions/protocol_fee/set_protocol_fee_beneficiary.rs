use generic_array_struct::generic_array_struct;

use crate::instructions::internal_utils::DiscmOnlyIxData;

// Accounts

#[generic_array_struct(builder pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct SetProtocolFeeBeneficiaryIxAccs<T> {
    /// The pool's current protocol fee beneficiary
    pub curr: T,

    /// New protocol fee beneficiary to set to
    pub new: T,

    /// The pool's state singleton PDA
    pub pool_state: T,
}

impl<T: Copy> SetProtocolFeeBeneficiaryIxAccs<T> {
    #[inline]
    pub const fn memset(val: T) -> Self {
        Self([val; SET_PROTOCOL_FEE_BENEFICIARY_IX_ACCS_LEN])
    }
}

pub type SetProtocolFeeBeneficiaryIxKeys<'a> = SetProtocolFeeBeneficiaryIxAccs<&'a [u8; 32]>;

pub type SetProtocolFeeBeneficiaryIxKeysOwned = SetProtocolFeeBeneficiaryIxAccs<[u8; 32]>;

pub type SetProtocolFeeBeneficiaryIxAccFlags = SetProtocolFeeBeneficiaryIxAccs<bool>;

pub const SET_PROTOCOL_FEE_BENEFICIARY_IX_IS_WRITER: SetProtocolFeeBeneficiaryIxAccFlags =
    SetProtocolFeeBeneficiaryIxAccFlags::memset(false).const_with_pool_state(true);

pub const SET_PROTOCOL_FEE_BENEFICIARY_IX_IS_SIGNER: SetProtocolFeeBeneficiaryIxAccFlags =
    SetProtocolFeeBeneficiaryIxAccFlags::memset(false).const_with_curr(true);

// Data

pub const SET_PROTOCOL_FEE_BENEFICIARY_IX_DISCM: u8 = 12;

pub type SetProtocolFeeBeneficiaryIxData = DiscmOnlyIxData<SET_PROTOCOL_FEE_BENEFICIARY_IX_DISCM>;

pub const SET_PROTOCOL_FEE_BENEFICIARY_IX_DATA_LEN: usize =
    SetProtocolFeeBeneficiaryIxData::DATA_LEN;
