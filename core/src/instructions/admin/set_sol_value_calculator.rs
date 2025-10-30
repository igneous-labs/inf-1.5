use core::{
    iter::{once, Chain, Once},
    slice,
};

use inf1_ctl_core::instructions::admin::set_sol_value_calculator::{
    SetSolValueCalculatorIxPreAccFlags, SetSolValueCalculatorIxPreKeysOwned,
    SET_SOL_VALUE_CALC_IX_PRE_IS_SIGNER, SET_SOL_VALUE_CALC_IX_PRE_IS_WRITER,
};
use inf1_svc_core::traits::SolValCalcAccs;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SetSolValueCalculatorIxAccs<T, I, C> {
    pub ix_prefix: I,
    pub calc_prog: T,
    pub calc: C,
}

pub type SetSolValueCalculatorAccsIter<'a, T> =
    Chain<Chain<slice::Iter<'a, T>, Once<&'a T>>, slice::Iter<'a, T>>;

impl<T, I: AsRef<[T]>, C: AsRef<[T]>> SetSolValueCalculatorIxAccs<T, I, C> {
    #[inline]
    pub fn seq(&self) -> SetSolValueCalculatorAccsIter<'_, T> {
        let Self {
            ix_prefix,
            calc_prog,
            calc,
        } = self;
        ix_prefix
            .as_ref()
            .iter()
            .chain(once(calc_prog))
            .chain(calc.as_ref())
    }
}

/// Call [`SetSolValueCalculatorIxAccs::seq`] on return value to create iterator
#[inline]
pub fn set_sol_value_calculator_ix_keys_owned<C: SolValCalcAccs>(
    SetSolValueCalculatorIxAccs {
        ix_prefix,
        calc_prog,
        calc,
    }: &SetSolValueCalculatorIxAccs<[u8; 32], SetSolValueCalculatorIxPreKeysOwned, C>,
) -> SetSolValueCalculatorIxAccs<[u8; 32], SetSolValueCalculatorIxPreKeysOwned, C::KeysOwned> {
    SetSolValueCalculatorIxAccs {
        ix_prefix: *ix_prefix,
        calc_prog: *calc_prog,
        calc: calc.suf_keys_owned(),
    }
}

/// Call [`SetSolValueCalculatorIxAccs::seq`] on return value to create iterator
#[inline]
pub fn set_sol_value_calculator_ix_is_signer<T, I, C: SolValCalcAccs>(
    SetSolValueCalculatorIxAccs { calc, .. }: &SetSolValueCalculatorIxAccs<T, I, C>,
) -> SetSolValueCalculatorIxAccs<bool, SetSolValueCalculatorIxPreAccFlags, C::AccFlags> {
    SetSolValueCalculatorIxAccs {
        ix_prefix: SET_SOL_VALUE_CALC_IX_PRE_IS_SIGNER,
        calc_prog: false,
        calc: calc.suf_is_signer(),
    }
}

/// Call [`SetSolValueCalculatorIxAccs::seq`] on return value to create iterator
#[inline]
pub fn set_sol_value_calculator_ix_is_writer<T, I, C: SolValCalcAccs>(
    SetSolValueCalculatorIxAccs { calc, .. }: &SetSolValueCalculatorIxAccs<T, I, C>,
) -> SetSolValueCalculatorIxAccs<bool, SetSolValueCalculatorIxPreAccFlags, C::AccFlags> {
    SetSolValueCalculatorIxAccs {
        ix_prefix: SET_SOL_VALUE_CALC_IX_PRE_IS_WRITER,
        calc_prog: false,
        calc: calc.suf_is_writer(),
    }
}
