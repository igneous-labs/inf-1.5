use inf1_core::instructions::swap::IxAccs;
use inf1_ctl_jiminy::instructions::swap::{v1::IxPreAccs, IxArgs};
use jiminy_cpi::{
    account::{Abr, AccountHandle},
    program_error::ProgramError,
};

use crate::{
    instructions::swap::v2::{SwapV2Ctl, SwapV2CtlIxAccounts},
    utils::{accs_split_first_chunk, split_suf_accs},
};

#[inline]
pub fn swap_split_v1_accs_into_v2<'a, 'acc>(
    abr: &Abr,
    accs: &'a [AccountHandle<'acc>],
    IxArgs {
        inp_lst_value_calc_accs,
        out_lst_value_calc_accs,
        ..
    }: &IxArgs,
) -> Result<SwapV2CtlIxAccounts<'a, 'acc>, ProgramError> {
    let (ix_prefix, suf) = accs_split_first_chunk(accs)?;
    let ix_prefix = IxPreAccs(*ix_prefix);

    let [(inp_calc_prog, inp_calc), (out_calc_prog, out_calc), (pricing_prog, pricing)] =
        split_suf_accs(suf, &[*inp_lst_value_calc_accs, *out_lst_value_calc_accs])?
            .map(|(prog, rest)| (*abr.get(prog).key(), rest));

    Ok(SwapV2Ctl::Swap(IxAccs {
        ix_prefix: ix_prefix.into(),
        inp_calc_prog,
        inp_calc,
        out_calc_prog,
        out_calc,
        pricing_prog,
        pricing,
    }))
}
