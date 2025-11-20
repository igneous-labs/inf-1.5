use std::path::Path;

use jiminy_program_error::ProgramError;
use mollusk_svm::{
    result::{Check, InstructionResult, ProgramResult},
    Mollusk,
};
use solana_account::Account;
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

use crate::{
    assert_prog_err_eq, test_fixtures_dir, workspace_root_dir, AccountMap,
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
/// - all programs in this workspace, excluding inf controller program
/// - spl token program
/// - associated token program
pub fn mollusk_inf_fixture_ctl() -> Mollusk {
    let mut svm = mollusk_with_token_progs();
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
        .chain(LOCAL_PROGRAMS.into_iter().filter_map(|(fname, key)| {
            if key == inf1_ctl_core::ID {
                None
            } else {
                Some((
                    workspace_root_dir()
                        .join("target/deploy")
                        .join(fname)
                        .with_extension("so"),
                    key,
                ))
            }
        }));
    mollusk_add_so_files(&mut svm, paths);
    svm
}

/// A mollusk instance with following programs all loaded in:
/// - all programs in test-fixtures/programs (NB: subdirs excluded), excluding inf controller program
/// - all programs in this workspace
/// - spl token program
/// - associated token program
pub fn mollusk_inf_local_ctl() -> Mollusk {
    let mut svm = mollusk_with_token_progs();
    let paths = FIXTURE_PROGRAMS
        .into_iter()
        .filter_map(|(fname, key)| {
            if key == inf1_ctl_core::ID {
                None
            } else {
                Some((
                    test_fixtures_dir()
                        .join("programs")
                        .join(fname)
                        .with_extension("so"),
                    key,
                ))
            }
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
    mollusk_add_so_files(&mut svm, paths);
    svm
}

pub fn mollusk_with_token_progs() -> Mollusk {
    let mut res = Mollusk::default();
    mollusk_svm_programs_token::token::add_program(&mut res);
    mollusk_svm_programs_token::associated_token::add_program(&mut res);
    res
}

/// All programs have owner = BPF_LOADER_UPGRADEABLE
pub fn mollusk_add_so_files(
    svm: &mut Mollusk,
    so_files: impl IntoIterator<Item = (impl AsRef<Path>, [u8; 32])>,
) {
    so_files.into_iter().for_each(|(path, key)| {
        svm.add_program_with_elf_and_loader(
            &key.into(),
            &std::fs::read(path).unwrap(),
            &BPF_LOADER_UPGRADEABLE_ADDR,
        );
    });
}

/// Returns `(accounts before, exec result)`
pub fn mollusk_exec(
    svm: &Mollusk,
    ix: &Instruction,
    onchain_state: &AccountMap,
) -> (AccountMap, InstructionResult) {
    let mut keys: Vec<_> = ix.accounts.iter().map(|a| a.pubkey).collect();
    keys.sort_unstable();
    keys.dedup();

    let accs_bef: AccountMap = keys
        .iter()
        .map(|k| {
            let (k, v) = onchain_state.get_key_value(k).unwrap();
            (*k, v.clone())
        })
        .collect();

    let mut accs_vec: Vec<_> = accs_bef.iter().map(|(k, v)| (*k, v.clone())).collect();
    accs_vec.sort_by_key(|(k, _)| *k);

    let res = svm.process_instruction(ix, &accs_vec);

    (accs_bef, res)
}

/// Like `mollusk_exec` but with validation checks applied to resulting accounts.
/// Returns `(accounts before, exec result)`.
pub fn mollusk_exec_validate(
    svm: &Mollusk,
    ix: &Instruction,
    onchain_state: &AccountMap,
    checks: &[Check],
) -> (AccountMap, InstructionResult) {
    let mut keys: Vec<_> = ix.accounts.iter().map(|a| a.pubkey).collect();
    keys.sort_unstable();
    keys.dedup();

    let accs_bef: AccountMap = keys
        .iter()
        .map(|k| {
            let (k, v) = onchain_state.get_key_value(k).unwrap();
            (*k, v.clone())
        })
        .collect();

    let mut accs_vec: Vec<_> = accs_bef.iter().map(|(k, v)| (*k, v.clone())).collect();
    accs_vec.sort_by_key(|(k, _)| *k);

    let res = svm.process_and_validate_instruction(ix, &accs_vec, checks);

    (accs_bef, res)
}

/// Returns `[bef, aft]`.
///
/// # Params
/// - `bef` should be `mollusk_exec(...).0`
/// - `aft` should be [`InstructionResult::resulting_accounts`] converted to AccountMap
pub fn acc_bef_aft<'a>(pk: &Pubkey, bef: &'a AccountMap, aft: &'a AccountMap) -> [&'a Account; 2] {
    let before = bef.get(pk).unwrap();
    let after = aft.get(pk).unwrap();
    [before, after]
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

pub fn assert_balanced(bef: &AccountMap, aft: &AccountMap) {
    let [lamports_bef, lamports_aft] = [bef, aft].map(|accounts| {
        accounts
            .values()
            .map(|acc| acc.lamports as u128)
            .sum::<u128>()
    });

    assert_eq!(lamports_bef, lamports_aft, "lamports not balanced");
}
