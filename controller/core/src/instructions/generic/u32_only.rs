use crate::instructions::internal_utils::caba;

pub const U32_IX_DATA_LEN: usize = 5;

/// Many instructions that operate on the program's lists just take a single u32
/// to represent list index as instruction args (after the discriminant).
/// This type generalizes their IxData type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct U32IxData<const DISCM: u8>([u8; U32_IX_DATA_LEN]);

// free fn for ease of use with generic type
// simple wrapper around primitive operation to ensure that
// type fn works the same as this free fn
/// Returns parsed u32 arg
#[inline]
pub const fn u32_ix_data_parse_no_discm(data: &[u8; 4]) -> u32 {
    u32::from_le_bytes(*data)
}

#[inline]
pub const fn new_u32_ix_data(discm: u8, arg: u32) -> [u8; U32_IX_DATA_LEN] {
    const A: usize = U32_IX_DATA_LEN;

    let mut d = [0u8; A];

    d = caba::<A, 0, 1>(d, &[discm]);
    d = caba::<A, 1, 4>(d, &arg.to_le_bytes());

    d
}

impl<const DISCM: u8> U32IxData<DISCM> {
    pub const DATA_LEN: usize = U32_IX_DATA_LEN;

    #[inline]
    pub const fn new(arg: u32) -> Self {
        Self(new_u32_ix_data(DISCM, arg))
    }

    #[inline]
    pub const fn as_buf(&self) -> &[u8; U32_IX_DATA_LEN] {
        &self.0
    }

    /// Returns parsed u32 arg
    #[inline]
    pub const fn parse_no_discm(data: &[u8; 4]) -> u32 {
        u32_ix_data_parse_no_discm(data)
    }
}
