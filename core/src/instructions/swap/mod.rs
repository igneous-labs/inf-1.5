use core::{iter::Chain, slice};

use inf1_ctl_core::instructions::swap as inf1_ctl_core_swap;
use inf1_svc_core::traits::SolValCalcAccs;

pub mod exact_in;
pub mod exact_out;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IxAccs<I, C, D, P> {
    pub ix_prefix: I,
    pub inp_calc: C,
    pub out_calc: D,
    pub pricing: P,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IxArgs<I, C, D, P> {
    pub inp_lst_index: u32,
    pub out_lst_index: u32,

    /// - min_amount_out for ExactIn
    /// - max_amount_in for ExactOut
    pub limit: u64,

    pub amount: u64,

    pub accs: IxAccs<I, C, D, P>,
}

impl<I, C: SolValCalcAccs, D: SolValCalcAccs, P> IxArgs<I, C, D, P> {
    #[inline]
    pub fn to_full(&self) -> inf1_ctl_core_swap::IxArgs {
        let Self {
            inp_lst_index,
            out_lst_index,
            limit,
            amount,
            accs: IxAccs {
                inp_calc, out_calc, ..
            },
        } = self;
        inf1_ctl_core_swap::IxArgs {
            // +1 for program account
            inp_lst_value_calc_accs: inp_calc.suf_len() + 1,
            out_lst_value_calc_accs: out_calc.suf_len() + 1,

            inp_lst_index: *inp_lst_index,
            out_lst_index: *out_lst_index,
            limit: *limit,
            amount: *amount,
        }
    }
}

pub type AccsIter<'a, T> = Chain<
    Chain<Chain<slice::Iter<'a, T>, slice::Iter<'a, T>>, slice::Iter<'a, T>>,
    slice::Iter<'a, T>,
>;

pub fn swap_ix_accs_seq<T, I: AsRef<[T]>, C: AsRef<[T]>, D: AsRef<[T]>, P: AsRef<[T]>>(
    IxAccs {
        ix_prefix,
        inp_calc,
        out_calc,
        pricing,
    }: &IxAccs<I, C, D, P>,
) -> AccsIter<'_, T> {
    ix_prefix
        .as_ref()
        .iter()
        .chain(inp_calc.as_ref())
        .chain(out_calc.as_ref())
        .chain(pricing.as_ref())
}
