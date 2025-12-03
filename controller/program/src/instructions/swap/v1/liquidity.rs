use inf1_core::instructions::swap::IxAccs;
use inf1_ctl_jiminy::instructions::{
    liquidity::{IxArgs, IxPreAccs},
    swap::{self, v2},
};
use jiminy_cpi::{
    account::{Abr, AccountHandle},
    program_error::ProgramError,
};

use crate::{
    instructions::swap::v2::{SwapV2Ctl, SwapV2CtlIxAccounts},
    utils::{accs_split_first_chunk, split_suf_accs},
};

#[inline]
pub fn conv_add_liq_args(
    IxArgs {
        lst_value_calc_accs,
        lst_index,
        amount,
        min_out,
    }: IxArgs,
) -> swap::IxArgs {
    swap::IxArgs {
        inp_lst_value_calc_accs: lst_value_calc_accs,
        inp_lst_index: lst_index,
        limit: min_out,
        amount,
        out_lst_index: u32::MAX,
        out_lst_value_calc_accs: 1,
    }
}

#[inline]
pub fn add_liq_split_v1_accs_into_v2<'a, 'acc>(
    abr: &Abr,
    accs: &'a [AccountHandle<'acc>],
    IxArgs {
        lst_value_calc_accs,
        ..
    }: &IxArgs,
) -> Result<SwapV2CtlIxAccounts<'a, 'acc>, ProgramError> {
    let (ix_prefix, suf) = accs_split_first_chunk(accs)?;
    let ix_prefix = IxPreAccs(*ix_prefix);

    let [(inp_calc_prog, inp_calc), (pricing_prog, pricing)] =
        split_suf_accs(suf, &[*lst_value_calc_accs])?
            .map(|(prog, rest)| (*abr.get(prog).key(), rest));

    Ok(SwapV2Ctl::AddLiq(IxAccs {
        ix_prefix: v2::IxPreAccs::clone_from_add_liq(&ix_prefix),
        inp_calc_prog,
        inp_calc,
        pricing_prog,
        pricing,
        // dont-care, unused
        // should be same as in SwapV2
        out_calc_prog: inf1_ctl_jiminy::ID,
        out_calc: &[],
    }))
}

#[inline]
pub fn conv_rem_liq_args(
    IxArgs {
        lst_value_calc_accs,
        lst_index,
        amount,
        min_out,
    }: IxArgs,
) -> swap::IxArgs {
    swap::IxArgs {
        out_lst_value_calc_accs: lst_value_calc_accs,
        out_lst_index: lst_index,
        limit: min_out,
        amount,
        inp_lst_index: u32::MAX,
        inp_lst_value_calc_accs: 1,
    }
}

#[inline]
pub fn rem_liq_split_v1_accs_into_v2<'a, 'acc>(
    abr: &Abr,
    accs: &'a [AccountHandle<'acc>],
    IxArgs {
        lst_value_calc_accs,
        ..
    }: &IxArgs,
) -> Result<SwapV2CtlIxAccounts<'a, 'acc>, ProgramError> {
    let (ix_prefix, suf) = accs_split_first_chunk(accs)?;
    let ix_prefix = IxPreAccs(*ix_prefix);

    let [(out_calc_prog, out_calc), (pricing_prog, pricing)] =
        split_suf_accs(suf, &[*lst_value_calc_accs])?
            .map(|(prog, rest)| (*abr.get(prog).key(), rest));

    Ok(SwapV2Ctl::AddLiq(IxAccs {
        ix_prefix: v2::IxPreAccs::clone_from_rem_liq(&ix_prefix),
        out_calc_prog,
        out_calc,
        pricing_prog,
        pricing,
        // dont-care, unused
        // should be same as in SwapV2
        inp_calc_prog: inf1_ctl_jiminy::ID,
        inp_calc: &[],
    }))
}
