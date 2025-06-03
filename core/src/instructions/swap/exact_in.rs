use inf1_ctl_core::instructions::swap::exact_in::{
    SwapExactInIxPreAccFlags, SwapExactInIxPreKeysOwned, SWAP_EXACT_IN_IX_PRE_IS_SIGNER,
    SWAP_EXACT_IN_IX_PRE_IS_WRITER,
};
use inf1_pp_core::traits::PriceExactInAccs;
use inf1_svc_core::traits::SolValCalcAccs;

use super::{IxAccs, IxArgs};

pub type SwapExactInIxAccs<I, C, D, P> = IxAccs<I, C, D, P>;

pub type SwapExactInIxArgs<C, D, P> = IxArgs<C, D, P>;

/// Use return value with [`super::swap_ix_accs_seq`] to create array
pub fn swap_exact_in_ix_keys_owned<C: SolValCalcAccs, D: SolValCalcAccs, P: PriceExactInAccs>(
    SwapExactInIxAccs {
        ix_prefix,
        pricing,
        inp_calc,
        out_calc,
    }: &SwapExactInIxAccs<SwapExactInIxPreKeysOwned, C, D, P>,
) -> SwapExactInIxAccs<SwapExactInIxPreKeysOwned, C::KeysOwned, D::KeysOwned, P::KeysOwned> {
    IxAccs {
        ix_prefix: *ix_prefix,
        inp_calc: inp_calc.suf_keys_owned(),
        out_calc: out_calc.suf_keys_owned(),
        pricing: pricing.suf_keys_owned(),
    }
}

/// Use return value with [`super::swap_ix_accs_seq`] to create array
pub fn swap_exact_in_ix_is_signer<I, C: SolValCalcAccs, D: SolValCalcAccs, P: PriceExactInAccs>(
    SwapExactInIxAccs {
        pricing,
        inp_calc,
        out_calc,
        ..
    }: &SwapExactInIxAccs<I, C, D, P>,
) -> SwapExactInIxAccs<SwapExactInIxPreAccFlags, C::AccFlags, D::AccFlags, P::AccFlags> {
    IxAccs {
        ix_prefix: SWAP_EXACT_IN_IX_PRE_IS_SIGNER,
        inp_calc: inp_calc.suf_is_signer(),
        out_calc: out_calc.suf_is_signer(),
        pricing: pricing.suf_is_signer(),
    }
}

/// Use return value with [`super::swap_ix_accs_seq`] to create array
pub fn swap_exact_in_ix_is_writer<I, C: SolValCalcAccs, D: SolValCalcAccs, P: PriceExactInAccs>(
    SwapExactInIxAccs {
        pricing,
        inp_calc,
        out_calc,
        ..
    }: &SwapExactInIxAccs<I, C, D, P>,
) -> SwapExactInIxAccs<SwapExactInIxPreAccFlags, C::AccFlags, D::AccFlags, P::AccFlags> {
    IxAccs {
        ix_prefix: SWAP_EXACT_IN_IX_PRE_IS_WRITER,
        inp_calc: inp_calc.suf_is_writer(),
        out_calc: out_calc.suf_is_writer(),
        pricing: pricing.suf_is_writer(),
    }
}
