use generic_array_struct::generic_array_struct;

use crate::{internal_utils::impl_memset, keys::CONST_KEYS_OWNED};

/// Order of these accounts are chosen s.t. they are the same as
/// that of `TokenInstruction::Mint`
#[generic_array_struct(all pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct MintClaimCoreAccs<T> {
    /// The pool's SOLS mint
    pub mint: T,

    /// User's SOLS token account to mint to when minting.
    /// User's (system) account to claim lamports to when claiming.
    pub out: T,

    /// SOLS pool user is minting/claiming from
    pub pool: T,
}

impl_memset!(MintClaimCoreAccs);

impl<T> MintClaimCoreAccs<T> {
    /// For better interop with type aliases
    #[inline]
    pub const fn new(a: [T; MINT_CLAIM_CORE_ACCS_LEN]) -> Self {
        Self(a)
    }

    /// SPL token instruction: Mint from `mint` to `out`, authorized by `pool`
    #[inline]
    pub const fn as_mint_to_token_ix(&self) -> &[T; MINT_CLAIM_CORE_ACCS_LEN] {
        &self.0
    }
}

/// Use by both `ClaimHolding` and `Swap`
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SwapArgs {
    /// minimum out amount for `ClaimHolding` and `Swap`
    pub limit: u64,
    pub amt: Option<u64>,
}

/// Constant accounts used for ClaimWrapped and BurnThenClaimWrapped
#[generic_array_struct(all pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct WrappedConstAccs<T> {
    /// Sysvar rent
    ///
    /// Needed to invoke SyncNative correctly
    pub rent: T,

    /// Tokenkeg program
    pub token: T,
}

pub type WrappedConstKeys<'a> = WrappedConstAccs<&'a [u8; 32]>;
pub type WrappedConstKeysOwned = WrappedConstAccs<[u8; 32]>;
pub type WrappedConstAccFlags = WrappedConstAccs<bool>;

impl_memset!(WrappedConstAccs);

pub const WRAPPED_CONST_IS_SIGNER: WrappedConstAccFlags = WrappedConstAccs::memset(false);
pub const WRAPPED_CONST_IS_WRITER: WrappedConstAccFlags = WrappedConstAccs::memset(false);

pub const WRAPPED_CONST_KEYS_OWNED: WrappedConstAccs<[u8; 32]> =
    WrappedConstAccs::const_from_destr(WrappedConstAccsDestr {
        rent: *CONST_KEYS_OWNED.sysvar_rent(),
        token: *CONST_KEYS_OWNED.tokenkeg_prog(),
    });
