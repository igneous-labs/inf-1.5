use std::collections::HashMap;

use jiminy_program_error::ProgramError;
use mollusk_svm::{
    result::{InstructionResult, ProgramResult},
    Mollusk,
};
use solana_account::Account;
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

use crate::{
    assert_prog_err_eq, test_fixtures_dir, workspace_root_dir, PkAccountTup,
    BPF_LOADER_UPGRADEABLE_ADDR, FIXTURE_PROGRAMS, LOCAL_PROGRAMS,
};

/// This needs to be ran outside the thread_local! static vars above
/// i.e. at the start of each proptest
/// or else it wont take effect
pub fn silence_mollusk_logs() {
    // set to level warn instead
    // of debug so we dont get overwhelmed by program logs
    // in our program proptests
    solana_logger::setup_with_default(
        "solana_rbpf::vm=warn,\
         solana_runtime::message_processor=warn,\
         solana_runtime::system_instruction_processor=warn",
    );
}

/// A mollusk instance with following programs all loaded in:
/// - all programs in test-fixtures/programs (NB: subdirs excluded)
/// - all programs in this workspace
/// - spl token program
/// - associated token program
pub fn mollusk_inf() -> Mollusk {
    let mut res = Mollusk::default();
    let paths = FIXTURE_PROGRAMS
        .into_iter()
        .map(|(fname, key)| {
            (
                test_fixtures_dir()
                    .join("programs")
                    .join(fname)
                    .with_extension("so"),
                key,
            )
        })
        .chain(LOCAL_PROGRAMS.into_iter().map(|(fname, key)| {
            (
                workspace_root_dir()
                    .join("target/deploy")
                    .join(fname)
                    .with_extension("so"),
                key,
            )
        }));
    paths.for_each(|(path, key)| {
        res.add_program_with_elf_and_loader(
            &key.into(),
            &std::fs::read(path).unwrap(),
            &BPF_LOADER_UPGRADEABLE_ADDR,
        );
    });
    mollusk_svm_programs_token::token::add_program(&mut res);
    mollusk_svm_programs_token::associated_token::add_program(&mut res);
    res
}

/// Returns `(accounts before, exec result)`
pub fn mollusk_exec(
    svm: &Mollusk,
    ix: &Instruction,
    onchain_state: &HashMap<Pubkey, Account>,
) -> (Vec<PkAccountTup>, InstructionResult) {
    let mut keys: Vec<_> = ix.accounts.iter().map(|a| a.pubkey).collect();
    keys.sort_unstable();
    keys.dedup();

    let accs_bef: Vec<_> = keys
        .iter()
        .map(|k| {
            let (k, v) = onchain_state.get_key_value(k).unwrap();
            (*k, v.clone())
        })
        .collect();

    let res = svm.process_instruction(ix, &accs_bef);

    (accs_bef, res)
}

/// Returns `[bef, aft]`.
///
/// # Params
/// - `bef` should be `mollusk_exec(...).0`
/// - `aft` should be [`InstructionResult::resulting_accounts`]
pub fn acc_bef_aft<'a>(
    pk: &Pubkey,
    bef: &'a [PkAccountTup],
    aft: &'a [PkAccountTup],
) -> [&'a Account; 2] {
    let i = bef.iter().position(|(p, _a)| pk == p).unwrap();
    let after = &aft[i];
    if after.0 != *pk {
        panic!("bef and aft not in same order");
    }
    [&bef[i].1, &after.1]
}

pub fn assert_jiminy_prog_err<E: Into<ProgramError>>(program_result: &ProgramResult, expected: E) {
    match program_result {
        ProgramResult::Failure(actual) => {
            assert_prog_err_eq(actual, &expected.into());
        }
        res => {
            panic!("Expected err but got: {res:#?}");
        }
    }
}
