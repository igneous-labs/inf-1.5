use crate::instructions::{
    common::PoolHolding,
    internal_utils::{csi_at, DiscmOnlyIxData},
    manager::{CrudHoldingIxPreAccs, CRUD_HOLDING_IX_PRE_IS_SIGNER, CRUD_HOLDING_IX_PRE_IS_WRITER},
};

// Accounts

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RemHoldingIxAccs<P, H, T> {
    /// [`CrudHoldingIxPreAccs`]
    pub pre: P,

    /// [`PoolHolding`]
    pub holding: H,

    /// holding's token program
    pub token_prog: T,
}

pub type RemHoldingIxGen<T> = RemHoldingIxAccs<CrudHoldingIxPreAccs<T>, PoolHolding<T>, T>;

pub const REM_HOLDING_IX_IS_WRITER: RemHoldingIxGen<bool> = RemHoldingIxGen {
    pre: CRUD_HOLDING_IX_PRE_IS_WRITER,
    holding: PoolHolding::memset(false).const_with_ata(true),
    token_prog: false,
};

pub const REM_HOLDING_IX_IS_SIGNER: RemHoldingIxGen<bool> = RemHoldingIxGen {
    pre: CRUD_HOLDING_IX_PRE_IS_SIGNER,
    holding: PoolHolding::memset(false),
    token_prog: false,
};

pub type RemHoldingIxAccsIter<'a, T> = csi_at!(@ @);

impl<T> RemHoldingIxGen<T> {
    #[inline]
    pub fn seq(&self) -> RemHoldingIxAccsIter<'_, T> {
        self.pre
            .0
            .iter()
            .chain(self.holding.0.iter())
            .chain(core::slice::from_ref(&self.token_prog))
    }
}

// Data

pub const REM_HOLDING_IX_DISCM: u8 = 14;

pub const REM_HOLDING_IX_DATA_LEN: usize = RemHoldingIxData::LEN;

pub type RemHoldingIxData = DiscmOnlyIxData<REM_HOLDING_IX_DISCM>;
