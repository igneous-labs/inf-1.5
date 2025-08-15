use crate::{
    instructions::{deprecated::lp::IxAccs, IxArgs, IxData},
    traits::main::PriceExactOutAccs,
};

use super::{IxPreAccs, IX_PRE_IS_SIGNER, IX_PRE_IS_WRITER};

// Accounts

/// `mint` is that of the input LST to add liquidity of
pub type PriceLpTokensToMintIxPreAccs<T> = IxPreAccs<T>;

pub type PriceLpTokensToMintIxPreKeys<'a> = PriceLpTokensToMintIxPreAccs<&'a [u8; 32]>;

pub type PriceLpTokensToMintIxPreKeysOwned = PriceLpTokensToMintIxPreAccs<[u8; 32]>;

pub type PriceLpTokensToMintIxPreAccFlags = PriceLpTokensToMintIxPreAccs<bool>;

pub const PRICE_LP_TOKENS_TO_MINT_IX_PRE_IS_WRITER: PriceLpTokensToMintIxPreAccFlags =
    IX_PRE_IS_WRITER;

pub const PRICE_LP_TOKENS_TO_MINT_IX_PRE_IS_SIGNER: PriceLpTokensToMintIxPreAccFlags =
    IX_PRE_IS_SIGNER;

// Data

/// amt - amount of input LST to add liquidity of
///
/// sol_value - sol value of `amt` input LST
pub type PriceLpTokensToMintIxArgs = IxArgs;

pub const PRICE_LP_TOKENS_TO_MINT_IX_DISCM: u8 = 2;

pub type PriceLpTokensToMintIxData = IxData<PRICE_LP_TOKENS_TO_MINT_IX_DISCM>;

// Combined accs

/// Call [`IxAccs::seq`] on return value to create iterator
pub fn price_lp_tokens_to_mint_ix_keys_owned<P: PriceExactOutAccs>(
    IxAccs { ix_prefix, suf }: &IxAccs<[u8; 32], P>,
) -> IxAccs<[u8; 32], P::KeysOwned> {
    IxAccs {
        ix_prefix: *ix_prefix,
        suf: suf.suf_keys_owned(),
    }
}

/// Call [`IxAccs::seq`] on return value to create iterator
pub fn price_lp_tokens_to_mint_ix_is_signer<T, P: PriceExactOutAccs>(
    IxAccs { ix_prefix: _, suf }: &IxAccs<T, P>,
) -> IxAccs<bool, P::AccFlags> {
    IxAccs {
        ix_prefix: PRICE_LP_TOKENS_TO_MINT_IX_PRE_IS_SIGNER,
        suf: suf.suf_is_signer(),
    }
}

/// Call [`IxAccs::seq`] on return value to create iterator
pub fn price_lp_tokens_to_mint_ix_is_writer<T, P: PriceExactOutAccs>(
    IxAccs { ix_prefix: _, suf }: &IxAccs<T, P>,
) -> IxAccs<bool, P::AccFlags> {
    IxAccs {
        ix_prefix: PRICE_LP_TOKENS_TO_MINT_IX_PRE_IS_WRITER,
        suf: suf.suf_is_writer(),
    }
}
