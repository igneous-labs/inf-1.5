pub const DISCM_ONLY_IX_DATA_LEN: usize = 1;

/// Many admin-facing instructions take no additional instruction args
/// apart from the ix discm. This type generalizes their IxData type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct DiscmOnlyIxData<const DISCM: u8>;

impl<const DISCM: u8> DiscmOnlyIxData<DISCM> {
    pub const DATA: u8 = DISCM;
    pub const DATA_LEN: usize = DISCM_ONLY_IX_DATA_LEN;

    #[inline]
    pub const fn as_buf() -> &'static [u8; DISCM_ONLY_IX_DATA_LEN] {
        &[Self::DATA]
    }
}
