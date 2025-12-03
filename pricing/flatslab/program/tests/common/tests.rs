use inf1_pp_flatslab_core::errs::FlatSlabProgramErr;
use inf1_pp_flatslab_program::CustomProgErr;
use inf1_test_utils::{assert_jiminy_prog_err, mollusk_exec, AccountMap};
use jiminy_entrypoint::program_error::ProgramError;
use solana_instruction::Instruction;

use crate::common::mollusk::SVM;

pub fn should_fail_with_flatslab_prog_err(
    ix: Instruction,
    accs: &AccountMap,
    expected: FlatSlabProgramErr,
) {
    should_fail_with_program_err(ix, accs, CustomProgErr(expected));
}

pub fn should_fail_with_program_err<E: Into<ProgramError>>(
    ix: Instruction,
    accs: &AccountMap,
    expected: E,
) {
    SVM.with(|mollusk| {
        let err = mollusk_exec(mollusk, &[ix], accs).unwrap_err();
        assert_jiminy_prog_err(&err, expected);
    });
}
