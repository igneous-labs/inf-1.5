use inf1_ctl_core::instructions::swap::v2::exact_out::{
    SwapExactOutV2IxPreAccFlags, SwapExactOutV2IxPreKeysOwned, SWAP_EXACT_OUT_V2_IX_PRE_IS_SIGNER,
    SWAP_EXACT_OUT_V2_IX_PRE_IS_WRITER,
};
use inf1_pp_core::traits::main::PriceExactOutAccs;
use inf1_svc_core::traits::SolValCalcAccs;

use crate::instructions::swap::{IxAccs, IxArgs};

pub type SwapExactOutIxAccs<T, I, C, D, P> = IxAccs<T, I, C, D, P>;

pub type SwapExactOutIxArgs<T, I, C, D, P> = IxArgs<T, I, C, D, P>;

/// Call [`IxAccs::seq`] on return value to create iterator
pub fn swap_exact_out_v2_ix_keys_owned<
    C: SolValCalcAccs,
    D: SolValCalcAccs,
    P: PriceExactOutAccs,
>(
    SwapExactOutIxAccs {
        ix_prefix,
        inp_calc_prog,
        inp_calc,
        out_calc_prog,
        out_calc,
        pricing_prog,
        pricing,
    }: &SwapExactOutIxAccs<[u8; 32], SwapExactOutV2IxPreKeysOwned, C, D, P>,
) -> SwapExactOutIxAccs<
    [u8; 32],
    SwapExactOutV2IxPreKeysOwned,
    C::KeysOwned,
    D::KeysOwned,
    P::KeysOwned,
> {
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

/// Call [`IxAccs::seq`] on return value to create iterator
pub fn swap_exact_out_v2_ix_is_signer<
    T,
    I,
    C: SolValCalcAccs,
    D: SolValCalcAccs,
    P: PriceExactOutAccs,
>(
    SwapExactOutIxAccs {
        pricing,
        inp_calc,
        out_calc,
        ..
    }: &SwapExactOutIxAccs<T, I, C, D, P>,
) -> SwapExactOutIxAccs<bool, SwapExactOutV2IxPreAccFlags, C::AccFlags, D::AccFlags, P::AccFlags> {
    IxAccs {
        ix_prefix: SWAP_EXACT_OUT_V2_IX_PRE_IS_SIGNER,
        inp_calc_prog: false,
        inp_calc: inp_calc.suf_is_signer(),
        out_calc_prog: false,
        out_calc: out_calc.suf_is_signer(),
        pricing_prog: false,
        pricing: pricing.suf_is_signer(),
    }
}

/// Call [`IxAccs::seq`] on return value to create iterator
pub fn swap_exact_out_v2_ix_is_writer<
    T,
    I,
    C: SolValCalcAccs,
    D: SolValCalcAccs,
    P: PriceExactOutAccs,
>(
    SwapExactOutIxAccs {
        pricing,
        inp_calc,
        out_calc,
        ..
    }: &SwapExactOutIxAccs<T, I, C, D, P>,
) -> SwapExactOutIxAccs<bool, SwapExactOutV2IxPreAccFlags, C::AccFlags, D::AccFlags, P::AccFlags> {
    IxAccs {
        ix_prefix: SWAP_EXACT_OUT_V2_IX_PRE_IS_WRITER,
        inp_calc_prog: false,
        inp_calc: inp_calc.suf_is_writer(),
        out_calc_prog: false,
        out_calc: out_calc.suf_is_writer(),
        pricing_prog: false,
        pricing: pricing.suf_is_writer(),
    }
}
