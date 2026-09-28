use crate::{
    accounts::ProtocolV1Fees,
    instructions::internal_utils::NanoPairIxData,
    typedefs::{Nanos, NanosOutOfRangeErr},
};

// Accounts: just SetFieldsIxAccs

// Data

pub const SET_PF_IX_DISCM: u8 = 22;

pub const SET_PF_IX_DATA_LEN: usize = SetPFIxData::LEN;

pub type SetPFIxData = NanoPairIxData<SET_PF_IX_DISCM>;

impl SetPFIxData {
    #[inline]
    pub const fn new(ProtocolV1Fees(data): &ProtocolV1Fees<Nanos>) -> Self {
        Self::new_checked(data)
    }

    #[inline]
    pub const fn parse_no_discm(
        data: &[u8; Self::LEN - 1],
    ) -> Result<ProtocolV1Fees<Nanos>, NanosOutOfRangeErr> {
        match Self::parse_no_discm_checked(data) {
            Err(e) => Err(e),
            Ok(x) => Ok(ProtocolV1Fees(x)),
        }
    }

    /// ## Returns
    /// - `None` if discm does not match
    /// - `Some(Err)` if either nano out of range
    #[inline]
    pub const fn parse(
        data: &[u8; Self::LEN],
    ) -> Option<Result<ProtocolV1Fees<Nanos>, NanosOutOfRangeErr>> {
        match Self::parse_checked(data) {
            None => None,
            Some(Err(e)) => Some(Err(e)),
            Some(Ok(x)) => Some(Ok(ProtocolV1Fees(x))),
        }
    }
}
