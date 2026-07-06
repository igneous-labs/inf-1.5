use super::{IxSufAccs, IX_SUF_IS_SIGNER, IX_SUF_IS_WRITER};

pub type SolToLstIxSufAccs<T> = IxSufAccs<T>;

pub type SolToLstIxSufKeys<'a> = SolToLstIxSufAccs<&'a [u8; 32]>;

pub type SolToLstIxSufKeysOwned = SolToLstIxSufAccs<[u8; 32]>;

pub type SolToLstIxSufAccFlags = SolToLstIxSufAccs<bool>;

pub const SOL_TO_LST_IX_SUF_IS_WRITER: SolToLstIxSufAccFlags = IX_SUF_IS_WRITER;

pub const SOL_TO_LST_IX_SUF_IS_SIGNER: SolToLstIxSufAccFlags = IX_SUF_IS_SIGNER;
