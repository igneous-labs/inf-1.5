use core::{
    iter::{once, Chain, Once},
    slice,
};

use inf1_ctl_core::instructions::swap as inf1_ctl_core_swap;
use inf1_svc_core::traits::SolValCalcAccs;

pub mod v1;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IxAccs<T, I, C, D, P> {
    pub ix_prefix: I,
    pub inp_calc_prog: T,
    pub inp_calc: C,
    pub out_calc_prog: T,
    pub out_calc: D,
    pub pricing_prog: T,
    pub pricing: P,
}

pub type AccsIter<'a, T> = Chain<
    Chain<
        Chain<
            Chain<Chain<Chain<slice::Iter<'a, T>, Once<&'a T>>, slice::Iter<'a, T>>, Once<&'a T>>,
            slice::Iter<'a, T>,
        >,
        Once<&'a T>,
    >,
    slice::Iter<'a, T>,
>;

impl<T, I: AsRef<[T]>, C: AsRef<[T]>, D: AsRef<[T]>, P: AsRef<[T]>> IxAccs<T, I, C, D, P> {
    #[inline]
    pub fn seq(&self) -> AccsIter<'_, T> {
        let Self {
            ix_prefix,
            inp_calc_prog,
            inp_calc,
            out_calc_prog,
            out_calc,
            pricing_prog,
            pricing,
        } = self;
        ix_prefix
            .as_ref()
            .iter()
            .chain(once(inp_calc_prog))
            .chain(inp_calc.as_ref())
            .chain(once(out_calc_prog))
            .chain(out_calc.as_ref())
            .chain(once(pricing_prog))
            .chain(pricing.as_ref())
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IxArgs<T, I, C, D, P> {
    pub inp_lst_index: u32,
    pub out_lst_index: u32,

    /// - min_amount_out for ExactIn
    /// - max_amount_in for ExactOut
    pub limit: u64,

    pub amount: u64,

    pub accs: IxAccs<T, I, C, D, P>,
}

impl<T, I, C: SolValCalcAccs, D: SolValCalcAccs, P> IxArgs<T, I, C, D, P> {
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
        let [inp_lst_value_calc_accs, out_lst_value_calc_accs] = [
            (inp_lst_index, inp_calc.suf_len()),
            (out_lst_index, out_calc.suf_len()),
        ]
        .map(|(i, l)| match *i {
            u32::MAX => 0,
            // +1 for program account
            _ => l + 1,
        });
        inf1_ctl_core_swap::IxArgs {
            inp_lst_value_calc_accs,
            out_lst_value_calc_accs,
            inp_lst_index: *inp_lst_index,
            out_lst_index: *out_lst_index,
            limit: *limit,
            amount: *amount,
        }
    }
}
