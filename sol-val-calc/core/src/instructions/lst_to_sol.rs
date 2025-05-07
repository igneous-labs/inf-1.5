// Accounts

use super::{internal_utils::caba, IxPreAccs, IX_PRE_IS_SIGNER, IX_PRE_IS_WRITER};

pub type LstToSolIxPreAccs<T> = IxPreAccs<T>;

pub type LstToSolIxPreKeys<'a> = LstToSolIxPreAccs<&'a [u8; 32]>;

pub type LstToSolIxPreKeysOwned = LstToSolIxPreAccs<[u8; 32]>;

pub type LstToSolIxPreAccFlags = LstToSolIxPreAccs<bool>;

pub const LST_TO_SOL_IX_PRE_IS_WRITER: LstToSolIxPreAccFlags = IX_PRE_IS_WRITER;

pub const LST_TO_SOL_IX_PRE_IS_SIGNER: LstToSolIxPreAccFlags = IX_PRE_IS_SIGNER;

// Data

pub const LST_TO_SOL_IX_DISCM: u8 = 0;

pub const LST_TO_SOL_IX_DATA_LEN: usize = 9;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LstToSolIxData([u8; LST_TO_SOL_IX_DATA_LEN]);

impl LstToSolIxData {
    #[inline]
    pub const fn new(lst: u64) -> Self {
        const A: usize = LST_TO_SOL_IX_DATA_LEN;

        let mut d = [0u8; A];
        d = caba::<A, 0, 1>(d, &[LST_TO_SOL_IX_DISCM]);
        d = caba::<A, 1, 8>(d, &lst.to_le_bytes());

        Self(d)
    }

    #[inline]
    pub const fn as_buf(&self) -> &[u8; LST_TO_SOL_IX_DATA_LEN] {
        &self.0
    }
}
