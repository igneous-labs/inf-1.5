use crate::{
    instructions::{price::IxAccs, IxArgs, IxData},
    traits::main::PriceExactInAccs,
};

use super::{IxPreAccs, IX_PRE_IS_SIGNER, IX_PRE_IS_WRITER};

// Accounts

pub type PriceExactInIxPreAccs<T> = IxPreAccs<T>;

pub type PriceExactInIxPreKeys<'a> = PriceExactInIxPreAccs<&'a [u8; 32]>;

pub type PriceExactInIxPreKeysOwned = PriceExactInIxPreAccs<[u8; 32]>;

pub type PriceExactInIxPreAccFlags = PriceExactInIxPreAccs<bool>;

pub const PRICE_EXACT_IN_IX_PRE_IS_WRITER: PriceExactInIxPreAccFlags = IX_PRE_IS_WRITER;

pub const PRICE_EXACT_IN_IX_PRE_IS_SIGNER: PriceExactInIxPreAccFlags = IX_PRE_IS_SIGNER;

// Data

/// amt - amount of input LST
///
/// sol_value - sol value of `amt` input LST
pub type PriceExactInIxArgs = IxArgs;

pub const PRICE_EXACT_IN_IX_DISCM: u8 = 0;

pub type PriceExactInIxData = IxData<PRICE_EXACT_IN_IX_DISCM>;

// Combined accs

/// Call [`IxAccs::seq`] on return value to create iterator
pub fn price_exact_in_ix_keys_owned<P: PriceExactInAccs>(
    IxAccs { ix_prefix, suf }: &IxAccs<[u8; 32], P>,
) -> IxAccs<[u8; 32], P::KeysOwned> {
    IxAccs {
        ix_prefix: *ix_prefix,
        suf: suf.suf_keys_owned(),
    }
}

/// Call [`IxAccs::seq`] on return value to create iterator
pub fn price_exact_in_ix_is_signer<T, P: PriceExactInAccs>(
    IxAccs { ix_prefix: _, suf }: &IxAccs<T, P>,
) -> IxAccs<bool, P::AccFlags> {
    IxAccs {
        ix_prefix: PRICE_EXACT_IN_IX_PRE_IS_SIGNER,
        suf: suf.suf_is_signer(),
    }
}

/// Call [`IxAccs::seq`] on return value to create iterator
pub fn price_exact_in_ix_is_writer<T, P: PriceExactInAccs>(
    IxAccs { ix_prefix: _, suf }: &IxAccs<T, P>,
) -> IxAccs<bool, P::AccFlags> {
    IxAccs {
        ix_prefix: PRICE_EXACT_IN_IX_PRE_IS_WRITER,
        suf: suf.suf_is_writer(),
    }
}
