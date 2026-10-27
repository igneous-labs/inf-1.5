use crate::instructions::internal_utils::{csi_at, DiscmOnlyIxData};

// Accounts

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InitProtocolIxAccs<P, S> {
    /// Protocol PDA
    pub protocol: P,

    /// system program
    pub system: S,
}

pub type InitProtocolIxGen<T> = InitProtocolIxAccs<T, T>;

pub const INIT_PROTOCOL_IX_IS_WRITER: InitProtocolIxGen<bool> = InitProtocolIxGen {
    protocol: true,
    system: false,
};

pub const INIT_PROTOCOL_IX_IS_SIGNER: InitProtocolIxGen<bool> = InitProtocolIxGen {
    protocol: false,
    system: false,
};

pub type InitProtocolIxAccsIter<'a, T> = csi_at!(@);

impl<T> InitProtocolIxGen<T> {
    #[inline]
    pub fn seq(&self) -> InitProtocolIxAccsIter<'_, T> {
        core::slice::from_ref(&self.protocol)
            .iter()
            .chain(core::slice::from_ref(&self.system).iter())
    }
}

// Data

pub const INIT_PROTOCOL_IX_DISCM: u8 = 30;

pub const INIT_PROTOCOL_IX_DATA_LEN: usize = InitProtocolIxData::LEN;

pub type InitProtocolIxData = DiscmOnlyIxData<INIT_PROTOCOL_IX_DISCM>;
