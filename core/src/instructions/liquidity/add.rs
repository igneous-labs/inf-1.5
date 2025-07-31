#![deprecated(
    since = "0.2.0",
    note = "Use SwapExactIn/Out with out_mint=LP token (INF) instead"
)]
#![allow(deprecated)]

use inf1_ctl_core::instructions::liquidity::add::{
    AddLiquidityIxPreAccFlags, AddLiquidityIxPreKeysOwned, ADD_LIQUIDITY_IX_PRE_IS_SIGNER,
    ADD_LIQUIDITY_IX_PRE_IS_WRITER,
};
use inf1_pp_core::traits::deprecated::PriceLpTokensToMintAccs;
use inf1_svc_core::traits::SolValCalcAccs;

use super::{IxAccs, IxArgs};

pub type AddLiquidityIxAccs<T, I, C, P> = IxAccs<T, I, C, P>;

pub type AddLiquidityIxArgs<T, I, C, P> = IxArgs<T, I, C, P>;

/// Call [`IxAccs::seq`] on return value to create iterator
pub fn add_liquidity_ix_keys_owned<C: SolValCalcAccs, P: PriceLpTokensToMintAccs>(
    AddLiquidityIxAccs {
        ix_prefix,
        lst_calc_prog,
        lst_calc,
        pricing_prog,
        pricing,
    }: &AddLiquidityIxAccs<[u8; 32], AddLiquidityIxPreKeysOwned, C, P>,
) -> AddLiquidityIxAccs<[u8; 32], AddLiquidityIxPreKeysOwned, C::KeysOwned, P::KeysOwned> {
    IxAccs {
        ix_prefix: *ix_prefix,
        lst_calc_prog: *lst_calc_prog,
        lst_calc: lst_calc.suf_keys_owned(),
        pricing_prog: *pricing_prog,
        pricing: pricing.suf_keys_owned(),
    }
}

/// Call [`IxAccs::seq`] on return value to create iterator
pub fn add_liquidity_ix_is_signer<T, I, C: SolValCalcAccs, P: PriceLpTokensToMintAccs>(
    AddLiquidityIxAccs {
        lst_calc, pricing, ..
    }: &AddLiquidityIxAccs<T, I, C, P>,
) -> AddLiquidityIxAccs<bool, AddLiquidityIxPreAccFlags, C::AccFlags, P::AccFlags> {
    IxAccs {
        ix_prefix: ADD_LIQUIDITY_IX_PRE_IS_SIGNER,
        lst_calc_prog: false,
        lst_calc: lst_calc.suf_is_signer(),
        pricing_prog: false,
        pricing: pricing.suf_is_signer(),
    }
}

/// Call [`IxAccs::seq`] on return value to create iterator
pub fn add_liquidity_ix_is_writer<T, I, C: SolValCalcAccs, P: PriceLpTokensToMintAccs>(
    AddLiquidityIxAccs {
        lst_calc, pricing, ..
    }: &AddLiquidityIxAccs<T, I, C, P>,
) -> AddLiquidityIxAccs<bool, AddLiquidityIxPreAccFlags, C::AccFlags, P::AccFlags> {
    IxAccs {
        ix_prefix: ADD_LIQUIDITY_IX_PRE_IS_WRITER,
        lst_calc_prog: false,
        lst_calc: lst_calc.suf_is_writer(),
        pricing_prog: false,
        pricing: pricing.suf_is_writer(),
    }
}
