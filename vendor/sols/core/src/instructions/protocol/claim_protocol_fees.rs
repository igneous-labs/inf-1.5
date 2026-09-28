use generic_array_struct::generic_array_struct;

use crate::{
    instructions::{
        internal_utils::{csi_at, DiscmOnlyIxData},
        user::MintClaimCoreAccs,
    },
    internal_utils::{impl_asref, impl_memset},
};

// Accounts

#[generic_array_struct(all pub)]
#[repr(transparent)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CPFIxPreAccs<T> {
    /// Protocol PDA
    pub protocol: T,

    /// Protocol fee beneficiary
    pub beneficiary: T,
}

impl_memset!(CPFIxPreAccs);
impl_asref!(CPFIxPreAccs);

pub type CPFIxPreKeys<'a> = CPFIxPreAccs<&'a [u8; 32]>;
pub type CPFIxPreKeysOwned = CPFIxPreAccs<[u8; 32]>;
pub type CPFIxPreAccFlags = CPFIxPreAccs<bool>;

pub const CPF_IX_PRE_IS_SIGNER: CPFIxPreAccFlags =
    CPFIxPreAccFlags::memset(false).const_with_beneficiary(true);

pub const CPF_IX_PRE_IS_WRITER: CPFIxPreAccFlags = CPFIxPreAccFlags::memset(false);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CPFIxAccs<P, M, T> {
    /// [`CPFIxPreAccs`]
    pub pre: P,

    /// [`MintClaimCoreAccs`]
    pub suf: M,

    /// token program
    pub token: T,
}

pub type CPFIxGen<T> = CPFIxAccs<CPFIxPreAccs<T>, MintClaimCoreAccs<T>, T>;

pub const CPF_IX_IS_SIGNER: CPFIxGen<bool> = CPFIxGen {
    pre: CPF_IX_PRE_IS_SIGNER,
    suf: MintClaimCoreAccs::memset(false),
    token: false,
};

pub const CPF_IX_IS_WRITER: CPFIxGen<bool> = CPFIxGen {
    pre: CPF_IX_PRE_IS_WRITER,
    suf: MintClaimCoreAccs::memset(true),
    token: false,
};

pub type CPFIxAccsIter<'a, T> = csi_at!(@ @);

impl<T> CPFIxGen<T> {
    #[inline]
    pub fn seq(&self) -> CPFIxAccsIter<'_, T> {
        self.pre
            .0
            .iter()
            .chain(self.suf.0.iter())
            .chain(core::slice::from_ref(&self.token))
    }
}

// Data

pub const CPF_IX_DISCM: u8 = 27;

pub const CPF_IX_DATA_LEN: usize = CPFIxData::LEN;

pub type CPFIxData = DiscmOnlyIxData<CPF_IX_DISCM>;
