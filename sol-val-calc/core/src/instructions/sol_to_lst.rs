// Accounts

use super::{internal_utils::caba, IxPreAccs, IX_PRE_IS_SIGNER, IX_PRE_IS_WRITER};

pub type SolToLstIxPreAccs<T> = IxPreAccs<T>;

pub type SolToLstIxPreKeys<'a> = SolToLstIxPreAccs<&'a [u8; 32]>;

pub type SolToLstIxPreKeysOwned = SolToLstIxPreAccs<[u8; 32]>;

pub type SolToLstIxPreAccFlags = SolToLstIxPreAccs<bool>;

pub const SOL_TO_LST_IX_PRE_IS_WRITER: SolToLstIxPreAccFlags = IX_PRE_IS_WRITER;

pub const SOL_TO_LST_IX_PRE_IS_SIGNER: SolToLstIxPreAccFlags = IX_PRE_IS_SIGNER;

// Data

pub const SOL_TO_LST_IX_DISCM: u8 = 1;

pub const SOL_TO_LST_IX_DATA_LEN: usize = 9;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SolToLstIxData([u8; SOL_TO_LST_IX_DATA_LEN]);

impl SolToLstIxData {
    #[inline]
    pub const fn new(lamports: u64) -> Self {
        const A: usize = SOL_TO_LST_IX_DATA_LEN;

        let mut d = [0u8; A];
        d = caba::<A, 0, 1>(d, &[SOL_TO_LST_IX_DISCM]);
        d = caba::<A, 1, 8>(d, &lamports.to_le_bytes());

        Self(d)
    }

    #[inline]
    pub const fn as_buf(&self) -> &[u8; SOL_TO_LST_IX_DATA_LEN] {
        &self.0
    }
}
