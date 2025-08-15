use crate::{
    instructions::{deprecated::lp::IxAccs, IxArgs, IxData},
    traits::main::PriceExactOutAccs,
};

use super::{IxPreAccs, IX_PRE_IS_SIGNER, IX_PRE_IS_WRITER};

// Accounts

/// `mint` is that of the output LST to remove liquidity of
pub type PriceLpTokensToRedeemIxPreAccs<T> = IxPreAccs<T>;

pub type PriceLpTokensToRedeemIxPreKeys<'a> = PriceLpTokensToRedeemIxPreAccs<&'a [u8; 32]>;

pub type PriceLpTokensToRedeemIxPreKeysOwned = PriceLpTokensToRedeemIxPreAccs<[u8; 32]>;

pub type PriceLpTokensToRedeemIxPreAccFlags = PriceLpTokensToRedeemIxPreAccs<bool>;

pub const PRICE_LP_TOKENS_TO_REDEEM_IX_PRE_IS_WRITER: PriceLpTokensToRedeemIxPreAccFlags =
    IX_PRE_IS_WRITER;

pub const PRICE_LP_TOKENS_TO_REDEEM_IX_PRE_IS_SIGNER: PriceLpTokensToRedeemIxPreAccFlags =
    IX_PRE_IS_SIGNER;

// Data

/// amt - amount of output LST to remove liquidity of
///
/// sol_value - sol value of `amt` output LST
pub type PriceLpTokensToRedeemIxArgs = IxArgs;

pub const PRICE_LP_TOKENS_TO_REDEEM_IX_DISCM: u8 = 3;

pub type PriceLpTokensToRedeemIxData = IxData<PRICE_LP_TOKENS_TO_REDEEM_IX_DISCM>;

// Combined accs

/// Call [`IxAccs::seq`] on return value to create iterator
pub fn price_lp_tokens_to_redeem_ix_keys_owned<P: PriceExactOutAccs>(
    IxAccs { ix_prefix, suf }: &IxAccs<[u8; 32], P>,
) -> IxAccs<[u8; 32], P::KeysOwned> {
    IxAccs {
        ix_prefix: *ix_prefix,
        suf: suf.suf_keys_owned(),
    }
}

/// Call [`IxAccs::seq`] on return value to create iterator
pub fn price_lp_tokens_to_redeem_ix_is_signer<T, P: PriceExactOutAccs>(
    IxAccs { ix_prefix: _, suf }: &IxAccs<T, P>,
) -> IxAccs<bool, P::AccFlags> {
    IxAccs {
        ix_prefix: PRICE_LP_TOKENS_TO_REDEEM_IX_PRE_IS_SIGNER,
        suf: suf.suf_is_signer(),
    }
}

/// Call [`IxAccs::seq`] on return value to create iterator
pub fn price_lp_tokens_to_redeem_ix_is_writer<T, P: PriceExactOutAccs>(
    IxAccs { ix_prefix: _, suf }: &IxAccs<T, P>,
) -> IxAccs<bool, P::AccFlags> {
    IxAccs {
        ix_prefix: PRICE_LP_TOKENS_TO_REDEEM_IX_PRE_IS_WRITER,
        suf: suf.suf_is_writer(),
    }
}
