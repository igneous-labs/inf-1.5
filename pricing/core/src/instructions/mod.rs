use internal_utils::caba;

mod internal_utils;

pub mod lp;
pub mod price;

// Data

pub const IX_DATA_LEN: usize = 17;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IxArgs {
    amt: u64,
    sol_value: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IxData<const DISCM: u8>([u8; IX_DATA_LEN]);

impl<const DISCM: u8> IxData<DISCM> {
    #[inline]
    pub const fn new(IxArgs { amt, sol_value }: IxArgs) -> Self {
        const A: usize = IX_DATA_LEN;

        let mut d = [0u8; A];

        d = caba::<A, 0, 1>(d, &[DISCM]);
        d = caba::<A, 1, 8>(d, &amt.to_le_bytes());
        d = caba::<A, 9, 8>(d, &sol_value.to_le_bytes());

        Self(d)
    }

    #[inline]
    pub const fn as_buf(&self) -> &[u8; IX_DATA_LEN] {
        &self.0
    }
}
