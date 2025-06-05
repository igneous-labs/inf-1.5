use serde::{Deserialize, Serialize};
use tsify_next::Tsify;

use crate::interface::B58PK;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct Instruction {
    #[tsify(type = "Uint8Array")] // Instead of number[]
    pub data: Box<[u8]>,
    pub accounts: Box<[AccountMeta]>,
    pub program_address: B58PK,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct AccountMeta {
    pub address: B58PK,

    /// Represents the role of an account in a transaction:
    /// - Readonly: 0
    /// - Writable: 1
    /// - ReadonlySigner: 2
    /// - WritableSigner: 3
    #[tsify(type = "0 | 1 | 2 | 3")]
    pub role: u8,
}

impl AccountMeta {
    pub(crate) const fn new(address: [u8; 32], role: Role) -> Self {
        Self {
            address: B58PK::new(address),
            role: role.to_u8(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Role {
    Readonly,
    Writable,
    ReadonlySigner,
    WritableSigner,
}

impl Role {
    pub(crate) const fn from_signer_writable(signer: bool, writable: bool) -> Self {
        match (signer, writable) {
            (true, true) => Self::WritableSigner,
            (true, false) => Self::ReadonlySigner,
            (false, true) => Self::Writable,
            (false, false) => Self::Readonly,
        }
    }

    pub(crate) const fn to_u8(self) -> u8 {
        match self {
            Self::Readonly => 0,
            Self::Writable => 1,
            Self::ReadonlySigner => 2,
            Self::WritableSigner => 3,
        }
    }
}

/// All 3 iterators must have the same length
pub(crate) fn keys_signer_writable_to_metas<'a>(
    keys: impl Iterator<Item = &'a [u8; 32]>,
    signer: impl Iterator<Item = &'a bool>,
    writable: impl Iterator<Item = &'a bool>,
) -> Box<[AccountMeta]> {
    keys.zip(signer)
        .zip(writable)
        .map(|((key, signer), writable)| {
            AccountMeta::new(*key, Role::from_signer_writable(*signer, *writable))
        })
        .collect()
}
