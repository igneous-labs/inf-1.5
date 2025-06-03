use inf1_ctl_core::instructions::liquidity::add::{
    AddLiquidityIxPreAccFlags, AddLiquidityIxPreKeysOwned, ADD_LIQUIDITY_IX_PRE_IS_SIGNER,
    ADD_LIQUIDITY_IX_PRE_IS_WRITER,
};
use inf1_pp_core::traits::PriceLpTokensToMintAccs;
use inf1_svc_core::traits::SolValCalcAccs;

use super::{IxAccs, IxArgs};

pub type AddLiquidityIxAccs<I, C, P> = IxAccs<I, C, P>;

pub type AddLiquidityIxArgs<C, P> = IxArgs<C, P>;

/// Use return value with [`super::liquidity_ix_accs_seq`] to create array
pub fn add_liquidity_ix_keys_owned<C: SolValCalcAccs, P: PriceLpTokensToMintAccs>(
    AddLiquidityIxAccs {
        ix_prefix,
        lst_calc,
        pricing,
    }: &AddLiquidityIxAccs<AddLiquidityIxPreKeysOwned, C, P>,
) -> AddLiquidityIxAccs<AddLiquidityIxPreKeysOwned, C::KeysOwned, P::KeysOwned> {
    IxAccs {
        ix_prefix: *ix_prefix,
        lst_calc: lst_calc.suf_keys_owned(),
        pricing: pricing.suf_keys_owned(),
    }
}

/// Use return value with [`super::liquidity_ix_accs_seq`] to create array
pub fn add_liquidity_ix_is_signer<I, C: SolValCalcAccs, P: PriceLpTokensToMintAccs>(
    AddLiquidityIxAccs {
        lst_calc, pricing, ..
    }: &AddLiquidityIxAccs<I, C, P>,
) -> AddLiquidityIxAccs<AddLiquidityIxPreAccFlags, C::AccFlags, P::AccFlags> {
    IxAccs {
        ix_prefix: ADD_LIQUIDITY_IX_PRE_IS_SIGNER,
        lst_calc: lst_calc.suf_is_signer(),
        pricing: pricing.suf_is_signer(),
    }
}

/// Use return value with [`super::liquidity_ix_accs_seq`] to create array
pub fn add_liquidity_ix_is_writer<I, C: SolValCalcAccs, P: PriceLpTokensToMintAccs>(
    AddLiquidityIxAccs {
        lst_calc, pricing, ..
    }: &AddLiquidityIxAccs<I, C, P>,
) -> AddLiquidityIxAccs<AddLiquidityIxPreAccFlags, C::AccFlags, P::AccFlags> {
    IxAccs {
        ix_prefix: ADD_LIQUIDITY_IX_PRE_IS_WRITER,
        lst_calc: lst_calc.suf_is_writer(),
        pricing: pricing.suf_is_writer(),
    }
}
