use inf1_ctl_core::instructions::swap::exact_in::{
    SwapExactInIxPreAccFlags, SwapExactInIxPreKeysOwned, SWAP_EXACT_IN_IX_PRE_IS_SIGNER,
    SWAP_EXACT_IN_IX_PRE_IS_WRITER,
};
use inf1_pp_core::traits::PriceExactInAccs;
use inf1_svc_core::traits::SolValCalcAccs;

use super::{IxAccs, IxArgs};

pub type SwapExactInIxAccs<T, I, C, D, P> = IxAccs<T, I, C, D, P>;

pub type SwapExactInIxArgs<T, I, C, D, P> = IxArgs<T, I, C, D, P>;

pub fn swap_exact_in_ix_keys_owned<C: SolValCalcAccs, D: SolValCalcAccs, P: PriceExactInAccs>(
    SwapExactInIxAccs {
        ix_prefix,
        inp_calc_prog,
        inp_calc,
        out_calc_prog,
        out_calc,
        pricing_prog,
        pricing,
    }: &SwapExactInIxAccs<[u8; 32], SwapExactInIxPreKeysOwned, C, D, P>,
) -> SwapExactInIxAccs<[u8; 32], SwapExactInIxPreKeysOwned, C::KeysOwned, D::KeysOwned, P::KeysOwned>
{
    IxAccs {
        ix_prefix: *ix_prefix,
        inp_calc_prog: *inp_calc_prog,
        inp_calc: inp_calc.suf_keys_owned(),
        out_calc_prog: *out_calc_prog,
        out_calc: out_calc.suf_keys_owned(),
        pricing_prog: *pricing_prog,
        pricing: pricing.suf_keys_owned(),
    }
}

pub fn swap_exact_in_ix_is_signer<
    T,
    I,
    C: SolValCalcAccs,
    D: SolValCalcAccs,
    P: PriceExactInAccs,
>(
    SwapExactInIxAccs {
        pricing,
        inp_calc,
        out_calc,
        ..
    }: &SwapExactInIxAccs<T, I, C, D, P>,
) -> SwapExactInIxAccs<bool, SwapExactInIxPreAccFlags, C::AccFlags, D::AccFlags, P::AccFlags> {
    IxAccs {
        ix_prefix: SWAP_EXACT_IN_IX_PRE_IS_SIGNER,
        inp_calc_prog: false,
        inp_calc: inp_calc.suf_is_signer(),
        out_calc_prog: false,
        out_calc: out_calc.suf_is_signer(),
        pricing_prog: false,
        pricing: pricing.suf_is_signer(),
    }
}

pub fn swap_exact_in_ix_is_writer<
    T,
    I,
    C: SolValCalcAccs,
    D: SolValCalcAccs,
    P: PriceExactInAccs,
>(
    SwapExactInIxAccs {
        pricing,
        inp_calc,
        out_calc,
        ..
    }: &SwapExactInIxAccs<T, I, C, D, P>,
) -> SwapExactInIxAccs<bool, SwapExactInIxPreAccFlags, C::AccFlags, D::AccFlags, P::AccFlags> {
    IxAccs {
        ix_prefix: SWAP_EXACT_IN_IX_PRE_IS_WRITER,
        inp_calc_prog: false,
        inp_calc: inp_calc.suf_is_writer(),
        out_calc_prog: false,
        out_calc: out_calc.suf_is_writer(),
        pricing_prog: false,
        pricing: pricing.suf_is_writer(),
    }
}
