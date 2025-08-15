use inf1_pp_flatslab_core::errs::FlatSlabProgramErr;
use inf1_pp_flatslab_program::CustomProgErr;
use jiminy_entrypoint::program_error::ProgramError;
use mollusk_svm::result::{InstructionResult, ProgramResult};
use solana_account::Account;
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

use crate::common::{mollusk::MOLLUSK, solana::assert_prog_err_eq};

pub fn should_fail_with_program_err<E: Into<ProgramError>>(
    ix: &Instruction,
    accs: &[(Pubkey, Account)],
    expected: E,
) {
    MOLLUSK.with(|mollusk| {
        let InstructionResult { program_result, .. } = mollusk.process_instruction(ix, accs);
        match program_result {
            ProgramResult::Failure(actual) => {
                assert_prog_err_eq(actual, expected.into());
            }
            res => {
                panic!("{res:#?}");
            }
        }
    });
}

pub fn should_fail_with_flatslab_prog_err(
    ix: &Instruction,
    accs: &[(Pubkey, Account)],
    expected: FlatSlabProgramErr,
) {
    should_fail_with_program_err(ix, accs, CustomProgErr(expected));
}
