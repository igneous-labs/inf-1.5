use crate::{
    instructions::{price::IxAccs, IxArgs, IxData},
    traits::main::PriceExactOutAccs,
};

use super::{IxPreAccs, IX_PRE_IS_SIGNER, IX_PRE_IS_WRITER};

// Accounts

pub type PriceExactOutIxPreAccs<T> = IxPreAccs<T>;

pub type PriceExactOutIxPreKeys<'a> = PriceExactOutIxPreAccs<&'a [u8; 32]>;

pub type PriceExactOutIxPreKeysOwned = PriceExactOutIxPreAccs<[u8; 32]>;

pub type PriceExactOutIxPreAccFlags = PriceExactOutIxPreAccs<bool>;

pub const PRICE_EXACT_OUT_IX_PRE_IS_WRITER: PriceExactOutIxPreAccFlags = IX_PRE_IS_WRITER;

pub const PRICE_EXACT_OUT_IX_PRE_IS_SIGNER: PriceExactOutIxPreAccFlags = IX_PRE_IS_SIGNER;

// Data

/// amt - amount of output LST
///
/// sol_value - sol value of `amt` output LST
pub type PriceExactOutIxArgs = IxArgs;

pub const PRICE_EXACT_OUT_IX_DISCM: u8 = 1;

pub type PriceExactOutIxData = IxData<PRICE_EXACT_OUT_IX_DISCM>;

// Combined accs

/// Call [`IxAccs::seq`] on return value to create iterator
pub fn price_exact_out_ix_keys_owned<P: PriceExactOutAccs>(
    IxAccs { ix_prefix, suf }: &IxAccs<[u8; 32], P>,
) -> IxAccs<[u8; 32], P::KeysOwned> {
    IxAccs {
        ix_prefix: *ix_prefix,
        suf: suf.suf_keys_owned(),
    }
}

/// Call [`IxAccs::seq`] on return value to create iterator
pub fn price_exact_out_ix_is_signer<T, P: PriceExactOutAccs>(
    IxAccs { ix_prefix: _, suf }: &IxAccs<T, P>,
) -> IxAccs<bool, P::AccFlags> {
    IxAccs {
        ix_prefix: PRICE_EXACT_OUT_IX_PRE_IS_SIGNER,
        suf: suf.suf_is_signer(),
    }
}

/// Call [`IxAccs::seq`] on return value to create iterator
pub fn price_exact_out_ix_is_writer<T, P: PriceExactOutAccs>(
    IxAccs { ix_prefix: _, suf }: &IxAccs<T, P>,
) -> IxAccs<bool, P::AccFlags> {
    IxAccs {
        ix_prefix: PRICE_EXACT_OUT_IX_PRE_IS_WRITER,
        suf: suf.suf_is_writer(),
    }
}
