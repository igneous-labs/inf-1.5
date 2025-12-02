use std::collections::HashMap;

use inf1_ctl_jiminy::{instructions::swap::v2::exact_out::SwapExactOutIxData, ID};
use inf1_pp_ag_core::instructions::PriceExactOutAccsAg;
use inf1_std::{
    instructions::swap::v2::exact_out::{
        swap_exact_out_v2_ix_is_signer, swap_exact_out_v2_ix_is_writer,
        swap_exact_out_v2_ix_keys_owned,
    },
    quote::Quote,
};
use inf1_test_utils::{
    assert_jiminy_prog_err, keys_signer_writable_to_metas, mock_prog_acc, mollusk_exec, AccountMap,
    ProgramDataAddr,
};
use jiminy_cpi::program_error::ProgramError;
use mollusk_svm::{
    result::{InstructionResult, ProgramResult},
    Mollusk,
};
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

use crate::{common::SVM, tests::swap::common::assert_correct_swap_exact_out};

mod add_liq;
mod rem_liq;
mod swap;

type Accs = super::super::Accs<PriceExactOutAccsAg>;
type Args = super::super::Args<PriceExactOutAccsAg>;

fn to_ix(args: &Args) -> Instruction {
    let accounts = keys_signer_writable_to_metas(
        swap_exact_out_v2_ix_keys_owned(&args.accs).seq(),
        swap_exact_out_v2_ix_is_signer(&args.accs).seq(),
        swap_exact_out_v2_ix_is_writer(&args.accs).seq(),
    );
    Instruction {
        program_id: Pubkey::new_from_array(ID),
        accounts,
        data: SwapExactOutIxData::new(&args.to_full()).as_buf().into(),
    }
}

fn add_prog_accs(
    am: &mut AccountMap,
    Accs {
        inp_calc_prog,
        out_calc_prog,
        pricing_prog,
        ..
    }: &Accs,
) {
    am.extend(
        [inp_calc_prog, out_calc_prog, pricing_prog]
            .into_iter()
            .map(|addr| {
                (
                    Pubkey::new_from_array(*addr),
                    // dont-care
                    mock_prog_acc(ProgramDataAddr::Raw(Default::default())),
                )
            }),
    );
}

/// Returns `None` if expected_err is `Some`
fn swap_exact_out_v2_test(
    svm: &Mollusk,
    args: &Args,
    bef: &AccountMap,
    expected_err: Option<impl Into<ProgramError>>,
) -> Option<Quote> {
    let ix = to_ix(args);
    let (
        _,
        InstructionResult {
            program_result,
            resulting_accounts,
            ..
        },
    ) = SVM.with(|svm| mollusk_exec(svm, &ix, bef));
    let aft: HashMap<_, _> = resulting_accounts.into_iter().collect();

    match expected_err {
        None => {
            assert_eq!(program_result, ProgramResult::Success);
            let clock = &svm.sysvars.clock;
            Some(assert_correct_swap_exact_out(
                bef,
                &aft,
                args,
                clock.epoch,
                clock.slot,
            ))
        }
        Some(e) => {
            assert_jiminy_prog_err(&program_result, e);
            None
        }
    }
}
