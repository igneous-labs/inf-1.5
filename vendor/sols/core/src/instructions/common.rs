use generic_array_struct::generic_array_struct;

use crate::{
    internal_utils::{addr_deref, const_map, impl_asref, impl_memset},
    keys::CONST_KEYS_OWNED,
};

#[generic_array_struct(all pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct MintPoolPair<T> {
    /// pool's SOLS mint
    pub mint: T,

    /// input SOLS pool
    pub pool: T,
}

pub type MintPoolPairKeys<'a> = MintPoolPair<&'a [u8; 32]>;
pub type MintPoolPairKeysOwned = MintPoolPair<[u8; 32]>;
pub type MintPoolPairAccFlags = MintPoolPair<bool>;

impl_memset!(MintPoolPair);

#[generic_array_struct(all pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct TokenSysProgs<T> {
    /// SPL token program
    pub token: T,

    /// System program
    pub system: T,
}

pub type TokenSysProgKeys<'a> = TokenSysProgs<&'a [u8; 32]>;
pub type TokenSysProgKeysOwned = TokenSysProgs<[u8; 32]>;
pub type TokenSysProgAccFlags = TokenSysProgs<bool>;

impl_memset!(TokenSysProgs);

/// Program accounts are always readonly
pub const TOKEN_SYS_PROG_IS_WRITER: TokenSysProgAccFlags = TokenSysProgAccFlags::memset(false);

/// Program accounts are always non-signers
pub const TOKEN_SYS_PROG_IS_SIGNER: TokenSysProgAccFlags = TokenSysProgAccFlags::memset(false);

// TODO: keeping this private for now bec there seems to be a weird bug
// onchain where this const gives [program_id(), tokenkeg_prog()] instead
// of [tokenkeg_prog(), system_prog()] unless we log the values
const TOKEN_SYS_PROG_KEYS: TokenSysProgKeys<'static> =
    TokenSysProgKeys::const_from_destr(TokenSysProgsDestr {
        token: CONST_KEYS_OWNED.tokenkeg_prog(),
        system: CONST_KEYS_OWNED.system_prog(),
    });

pub const TOKEN_SYS_PROG_KEYS_OWNED: TokenSysProgKeysOwned =
    TokenSysProgs(const_map!([0; 32], TOKEN_SYS_PROG_KEYS.0, addr_deref));

#[generic_array_struct(all pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct PoolHolding<T> {
    /// Pools' associated token account of `mint`
    pub ata: T,

    /// This holding's mint
    pub mint: T,
}

pub type PoolHoldingKeys<'a> = PoolHolding<&'a [u8; 32]>;
pub type PoolHoldingKeysOwned = PoolHolding<[u8; 32]>;
pub type PoolHoldingAccFlags = PoolHolding<bool>;

impl_memset!(PoolHolding);
impl_asref!(PoolHolding);

#[generic_array_struct(all pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct HoldingProgAccs<T> {
    /// SPL token program
    pub token: T,

    /// SOL value calculator program
    pub svc: T,
}

pub type HoldingProgKeys<'a> = HoldingProgAccs<&'a [u8; 32]>;
pub type HoldingProgKeysOwned = HoldingProgAccs<[u8; 32]>;
pub type HoldingProgAccFlags = HoldingProgAccs<bool>;

impl_memset!(HoldingProgAccs);
impl_asref!(HoldingProgAccs);

pub const HOLDING_PROG_IS_WRITER: HoldingProgAccFlags = HoldingProgAccFlags::memset(false);

pub const HOLDING_PROG_IS_SIGNER: HoldingProgAccFlags = HoldingProgAccFlags::memset(false);

#[generic_array_struct(all pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct SvcProgWhitelistAccs<T> {
    /// The SOL value calculator program
    pub prog: T,

    /// Protocol PDA
    pub protocol: T,
}

pub type SvcProgWhitelistKeys<'a> = SvcProgWhitelistAccs<&'a [u8; 32]>;
pub type SvcProgWhitelistKeysOwned = SvcProgWhitelistAccs<[u8; 32]>;
pub type SvcProgWhitelistAccFlags = SvcProgWhitelistAccs<bool>;

impl_memset!(SvcProgWhitelistAccs);
impl_asref!(SvcProgWhitelistAccs);

#[generic_array_struct(all pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct SetAuthIxAccs<T> {
    /// The data account that contains the authority to be changed
    pub data: T,

    /// Old authority
    pub old: T,

    /// New authority to change to
    pub new: T,
}

pub type SetAuthIxKeys<'a> = SetAuthIxAccs<&'a [u8; 32]>;
pub type SetAuthIxKeysOwned = SetAuthIxAccs<[u8; 32]>;
pub type SetAuthIxAccFlags = SetAuthIxAccs<bool>;

impl_memset!(SetAuthIxAccs);

pub const SET_AUTH_IX_IS_WRITER: SetAuthIxAccFlags =
    SetAuthIxAccFlags::memset(false).const_with_data(true);

pub const SET_AUTH_IX_IS_SIGNER_NEW_AUTH_NO_SIGN: SetAuthIxAccFlags =
    SetAuthIxAccFlags::memset(false).const_with_old(true);

pub const SET_AUTH_IX_IS_SIGNER_NEW_AUTH_MUST_SIGN: SetAuthIxAccFlags =
    SetAuthIxAccFlags::memset(true).const_with_data(false);

#[generic_array_struct(all pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct SetFieldsIxAccs<T> {
    /// The data account that contains the fields to be changed
    pub data: T,

    /// Authority authorized to set these fields
    pub auth: T,
}

pub type SetFieldsIxKeys<'a> = SetFieldsIxAccs<&'a [u8; 32]>;
pub type SetFieldsIxKeysOwned = SetFieldsIxAccs<[u8; 32]>;
pub type SetFieldsIxAccFlags = SetFieldsIxAccs<bool>;

impl_memset!(SetFieldsIxAccs);

pub const SET_FIELDS_IX_IS_WRITER: SetFieldsIxAccFlags =
    SetFieldsIxAccFlags::memset(false).const_with_data(true);

pub const SET_FIELDS_IX_IS_SIGNER: SetFieldsIxAccFlags =
    SetFieldsIxAccFlags::memset(false).const_with_auth(true);
