use inf1_ctl_core::instructions::liquidity::remove::{
    RemoveLiquidityIxPreAccFlags, RemoveLiquidityIxPreKeysOwned, REMOVE_LIQUIDITY_IX_PRE_IS_SIGNER,
    REMOVE_LIQUIDITY_IX_PRE_IS_WRITER,
};
use inf1_pp_core::traits::PriceLpTokensToRedeemAccs;
use inf1_svc_core::traits::SolValCalcAccs;

use super::{IxAccs, IxArgs};

pub type RemoveLiquidityIxAccs<T, I, C, P> = IxAccs<T, I, C, P>;

pub type RemoveLiquidityIxArgs<T, I, C, P> = IxArgs<T, I, C, P>;

pub fn remove_liquidity_ix_keys_owned<C: SolValCalcAccs, P: PriceLpTokensToRedeemAccs>(
    RemoveLiquidityIxAccs {
        ix_prefix,
        lst_calc_prog,
        lst_calc,
        pricing_prog,
        pricing,
    }: &RemoveLiquidityIxAccs<[u8; 32], RemoveLiquidityIxPreKeysOwned, C, P>,
) -> RemoveLiquidityIxAccs<[u8; 32], RemoveLiquidityIxPreKeysOwned, C::KeysOwned, P::KeysOwned> {
    IxAccs {
        ix_prefix: *ix_prefix,
        lst_calc_prog: *lst_calc_prog,
        lst_calc: lst_calc.suf_keys_owned(),
        pricing_prog: *pricing_prog,
        pricing: pricing.suf_keys_owned(),
    }
}

pub fn remove_liquidity_ix_is_signer<T, I, C: SolValCalcAccs, P: PriceLpTokensToRedeemAccs>(
    RemoveLiquidityIxAccs {
        lst_calc, pricing, ..
    }: &RemoveLiquidityIxAccs<T, I, C, P>,
) -> RemoveLiquidityIxAccs<bool, RemoveLiquidityIxPreAccFlags, C::AccFlags, P::AccFlags> {
    IxAccs {
        ix_prefix: REMOVE_LIQUIDITY_IX_PRE_IS_SIGNER,
        lst_calc_prog: false,
        lst_calc: lst_calc.suf_is_signer(),
        pricing_prog: false,
        pricing: pricing.suf_is_signer(),
    }
}

pub fn remove_liquidity_ix_is_writer<T, I, C: SolValCalcAccs, P: PriceLpTokensToRedeemAccs>(
    RemoveLiquidityIxAccs {
        lst_calc, pricing, ..
    }: &RemoveLiquidityIxAccs<T, I, C, P>,
) -> RemoveLiquidityIxAccs<bool, RemoveLiquidityIxPreAccFlags, C::AccFlags, P::AccFlags> {
    IxAccs {
        ix_prefix: REMOVE_LIQUIDITY_IX_PRE_IS_WRITER,
        lst_calc_prog: false,
        lst_calc: lst_calc.suf_is_writer(),
        pricing_prog: false,
        pricing: pricing.suf_is_writer(),
    }
}
