use super::{IxAccsGen, IxSufAccs, IX_IS_SIGNER, IX_IS_WRITER, IX_SUF_IS_SIGNER, IX_SUF_IS_WRITER};

pub type LstToSolIxSufAccs<T> = IxSufAccs<T>;

pub type LstToSolIxSufKeys<'a> = LstToSolIxSufAccs<&'a [u8; 32]>;
pub type LstToSolIxSufKeysOwned = LstToSolIxSufAccs<[u8; 32]>;
pub type LstToSolIxSufAccFlags = LstToSolIxSufAccs<bool>;

pub const LST_TO_SOL_IX_SUF_IS_WRITER: LstToSolIxSufAccFlags = IX_SUF_IS_WRITER;

pub const LST_TO_SOL_IX_SUF_IS_SIGNER: LstToSolIxSufAccFlags = IX_SUF_IS_SIGNER;

pub type LstToSolIxAccsGen<T> = IxAccsGen<T>;

pub type LstToSolIxKeys<'a> = LstToSolIxAccsGen<&'a [u8; 32]>;
pub type LstToSolIxKeysOwned = LstToSolIxAccsGen<[u8; 32]>;
pub type LstToSolIxAccFlags = LstToSolIxAccsGen<bool>;

pub const LST_TO_SOL_IX_IS_WRITER: LstToSolIxAccFlags = IX_IS_WRITER;

pub const LST_TO_SOL_IX_IS_SIGNER: LstToSolIxAccFlags = IX_IS_SIGNER;
