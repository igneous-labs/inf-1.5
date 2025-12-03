use inf1_ctl_jiminy::{instructions::swap::v2::exact_in::SwapExactInIxData, ID};
use inf1_pp_ag_core::instructions::PriceExactInAccsAg;
use inf1_std::{
    instructions::swap::v2::exact_in::{
        swap_exact_in_v2_ix_is_signer, swap_exact_in_v2_ix_is_writer,
        swap_exact_in_v2_ix_keys_owned,
    },
    quote::Quote,
};
use inf1_test_utils::{
    assert_jiminy_prog_err, keys_signer_writable_to_metas, mollusk_exec, AccountMap,
};
use jiminy_cpi::program_error::ProgramError;
use mollusk_svm::Mollusk;
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

use crate::tests::swap::common::assert_correct_swap_exact_in;

mod add_liq;
mod rem_liq;
mod swap;

type Accs = super::super::Accs<PriceExactInAccsAg>;
type Args = super::super::Args<PriceExactInAccsAg>;

fn to_ix(args: &Args) -> Instruction {
    let accounts = keys_signer_writable_to_metas(
        swap_exact_in_v2_ix_keys_owned(&args.accs).seq(),
        swap_exact_in_v2_ix_is_signer(&args.accs).seq(),
        swap_exact_in_v2_ix_is_writer(&args.accs).seq(),
    );
    Instruction {
        program_id: Pubkey::new_from_array(ID),
        accounts,
        data: SwapExactInIxData::new(&args.to_full()).as_buf().into(),
    }
}

/// Returns `None` if expected_err is `Some`
fn swap_exact_in_v2_test(
    svm: &Mollusk,
    args: &Args,
    bef: &AccountMap,
    expected_err: Option<impl Into<ProgramError>>,
) -> Option<Quote> {
    let ix = to_ix(args);

    let result = mollusk_exec(svm, &[ix], bef);

    match expected_err {
        None => {
            let aft = result.unwrap().resulting_accounts;
            let clock = &svm.sysvars.clock;
            Some(assert_correct_swap_exact_in(
                bef,
                &aft,
                args,
                clock.epoch,
                clock.slot,
            ))
        }
        Some(e) => {
            assert_jiminy_prog_err(&result.unwrap_err(), e);
            None
        }
    }
}
