use generic_array_struct::generic_array_struct;

use crate::{
    instructions::internal_utils::NanoPairIxData,
    typedefs::{Nanos, NanosOutOfRangeErr},
};

// Accounts: just SetFieldsIxAccs

// Data

pub const SET_RRR_IX_DISCM: u8 = 20;

pub const SET_RRR_IX_DATA_LEN: usize = SetRRRIxData::LEN;

pub type SetRRRIxData = NanoPairIxData<SET_RRR_IX_DISCM>;

#[generic_array_struct(all pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct RRR<T> {
    pub floor: T,
    pub ceil: T,
}

pub type RRRNanos = RRR<Nanos>;

impl SetRRRIxData {
    #[inline]
    pub const fn new(RRR(data): &RRRNanos) -> Self {
        Self::new_checked(data)
    }

    #[inline]
    pub const fn parse_no_discm(
        data: &[u8; Self::LEN - 1],
    ) -> Result<RRRNanos, NanosOutOfRangeErr> {
        match Self::parse_no_discm_checked(data) {
            Err(e) => Err(e),
            Ok(x) => Ok(RRR(x)),
        }
    }

    /// ## Returns
    /// - `None` if discm does not match
    /// - `Some(Err)` if either nano out of range
    #[inline]
    pub const fn parse(data: &[u8; Self::LEN]) -> Option<Result<RRRNanos, NanosOutOfRangeErr>> {
        match Self::parse_checked(data) {
            None => None,
            Some(Err(e)) => Some(Err(e)),
            Some(Ok(x)) => Some(Ok(RRR(x))),
        }
    }
}
