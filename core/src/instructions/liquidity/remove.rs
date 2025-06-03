use inf1_ctl_core::instructions::liquidity::remove::{
    RemoveLiquidityIxPreAccFlags, RemoveLiquidityIxPreKeysOwned, REMOVE_LIQUIDITY_IX_PRE_IS_SIGNER,
    REMOVE_LIQUIDITY_IX_PRE_IS_WRITER,
};
use inf1_pp_core::traits::PriceLpTokensToRedeemAccs;
use inf1_svc_core::traits::SolValCalcAccs;

use super::{IxAccs, IxArgs};

pub type RemoveLiquidityIxAccs<I, C, P> = IxAccs<I, C, P>;

pub type RemoveLiquidityIxArgs<C, P> = IxArgs<C, P>;

/// Use return value with [`super::accs_seq`] to create array
pub fn remove_liquidity_keys_owned<C: SolValCalcAccs, P: PriceLpTokensToRedeemAccs>(
    RemoveLiquidityIxAccs {
        ix_prefix,
        lst_calc,
        pricing,
    }: &RemoveLiquidityIxAccs<RemoveLiquidityIxPreKeysOwned, C, P>,
) -> RemoveLiquidityIxAccs<RemoveLiquidityIxPreKeysOwned, C::KeysOwned, P::KeysOwned> {
    IxAccs {
        ix_prefix: *ix_prefix,
        lst_calc: lst_calc.suf_keys_owned(),
        pricing: pricing.suf_keys_owned(),
    }
}

/// Use return value with [`super::accs_seq`] to create array
pub fn remove_liquidity_is_signer<I, C: SolValCalcAccs, P: PriceLpTokensToRedeemAccs>(
    RemoveLiquidityIxAccs {
        lst_calc, pricing, ..
    }: &RemoveLiquidityIxAccs<I, C, P>,
) -> RemoveLiquidityIxAccs<RemoveLiquidityIxPreAccFlags, C::AccFlags, P::AccFlags> {
    IxAccs {
        ix_prefix: REMOVE_LIQUIDITY_IX_PRE_IS_SIGNER,
        lst_calc: lst_calc.suf_is_signer(),
        pricing: pricing.suf_is_signer(),
    }
}

/// Use return value with [`super::accs_seq`] to create array
pub fn remove_liquidity_is_writer<I, C: SolValCalcAccs, P: PriceLpTokensToRedeemAccs>(
    RemoveLiquidityIxAccs {
        lst_calc, pricing, ..
    }: &RemoveLiquidityIxAccs<I, C, P>,
) -> RemoveLiquidityIxAccs<RemoveLiquidityIxPreAccFlags, C::AccFlags, P::AccFlags> {
    IxAccs {
        ix_prefix: REMOVE_LIQUIDITY_IX_PRE_IS_WRITER,
        lst_calc: lst_calc.suf_is_writer(),
        pricing: pricing.suf_is_writer(),
    }
}
