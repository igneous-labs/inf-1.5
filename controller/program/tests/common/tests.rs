use inf1_core::inf1_ctl_core::err::Inf1CtlErr;
use inf1_ctl_jiminy::program_err::Inf1CtlCustomProgErr;
use inf1_test_utils::assert_jiminy_prog_err;
use jiminy_entrypoint::program_error::ProgramError;
use mollusk_svm::result::InstructionResult;
use solana_account::Account;
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

use crate::common::mollusk::SVM;

pub fn should_fail_with_inf1_ctl_prog_err(
    ix: &Instruction,
    accs: &[(Pubkey, Account)],
    expected: Inf1CtlErr,
) {
    should_fail_with_program_err(ix, accs, Inf1CtlCustomProgErr(expected));
}

pub fn should_fail_with_program_err<E: Into<ProgramError>>(
    ix: &Instruction,
    accs: &[(Pubkey, Account)],
    expected: E,
) {
    SVM.with(|mollusk| {
        let InstructionResult { program_result, .. } = mollusk.process_instruction(ix, accs);
        assert_jiminy_prog_err(&program_result, expected);
    });
}
