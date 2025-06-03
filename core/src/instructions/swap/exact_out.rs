use inf1_ctl_core::instructions::swap::exact_out::{
    SwapExactOutIxPreAccFlags, SwapExactOutIxPreKeysOwned, SWAP_EXACT_OUT_IX_PRE_IS_SIGNER,
    SWAP_EXACT_OUT_IX_PRE_IS_WRITER,
};
use inf1_pp_core::traits::PriceExactOutAccs;
use inf1_svc_core::traits::SolValCalcAccs;

use super::{IxAccs, IxArgs};

pub type SwapExactOutIxAccs<I, C, D, P> = IxAccs<I, C, D, P>;

pub type SwapExactOutIxArgs<I, C, D, P> = IxArgs<I, C, D, P>;

/// Use return value with [`super::swap_ix_accs_seq`] to create array
pub fn swap_exact_out_ix_keys_owned<C: SolValCalcAccs, D: SolValCalcAccs, P: PriceExactOutAccs>(
    SwapExactOutIxAccs {
        ix_prefix,
        pricing,
        inp_calc,
        out_calc,
    }: &SwapExactOutIxAccs<SwapExactOutIxPreKeysOwned, C, D, P>,
) -> SwapExactOutIxAccs<SwapExactOutIxPreKeysOwned, C::KeysOwned, D::KeysOwned, P::KeysOwned> {
    IxAccs {
        ix_prefix: *ix_prefix,
        inp_calc: inp_calc.suf_keys_owned(),
        out_calc: out_calc.suf_keys_owned(),
        pricing: pricing.suf_keys_owned(),
    }
}

/// Use return value with [`super::swap_ix_accs_seq`] to create array
pub fn swap_exact_out_ix_is_signer<
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
    }: &SwapExactOutIxAccs<I, C, D, P>,
) -> SwapExactOutIxAccs<SwapExactOutIxPreAccFlags, C::AccFlags, D::AccFlags, P::AccFlags> {
    IxAccs {
        ix_prefix: SWAP_EXACT_OUT_IX_PRE_IS_SIGNER,
        inp_calc: inp_calc.suf_is_signer(),
        out_calc: out_calc.suf_is_signer(),
        pricing: pricing.suf_is_signer(),
    }
}

/// Use return value with [`super::swap_ix_accs_seq`] to create array
pub fn swap_exact_out_ix_is_writer<
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
    }: &SwapExactOutIxAccs<I, C, D, P>,
) -> SwapExactOutIxAccs<SwapExactOutIxPreAccFlags, C::AccFlags, D::AccFlags, P::AccFlags> {
    IxAccs {
        ix_prefix: SWAP_EXACT_OUT_IX_PRE_IS_WRITER,
        inp_calc: inp_calc.suf_is_writer(),
        out_calc: out_calc.suf_is_writer(),
        pricing: pricing.suf_is_writer(),
    }
}
