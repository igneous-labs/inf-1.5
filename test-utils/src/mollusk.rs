use std::path::Path;

use jiminy_program_error::ProgramError as JiminyProgramError;
use mollusk_svm::{
    result::{Check, ProgramResult},
    Mollusk,
};
use solana_account::Account;
use solana_instruction::{error::InstructionError, Instruction};
use solana_program_error::ProgramError;
use solana_pubkey::Pubkey;

use crate::{
    assert_prog_err_eq, test_fixtures_dir, workspace_root_dir, AccountMap,
    BPF_LOADER_UPGRADEABLE_ADDR, FIXTURE_PROGRAMS, LOCAL_PROGRAMS,
};

/// Successful execution result containing accounts after execution.
#[derive(Clone, Debug)]
pub struct ExecOk {
    pub resulting_accounts: AccountMap,
    pub compute_units_consumed: u64,
    pub execution_time: u64,
    pub return_data: Vec<u8>,
}

#[derive(Clone, Debug)]
pub enum ExecErr {
    Failure(ProgramError),
    UnknownError(InstructionError),
}

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

/// On success:
/// - returns `Ok(ExecOk)` with resulting accounts as `AccountMap`
/// - asserts lamports are balanced
/// - asserts all resulting accounts are rent-exempt
///
/// On failure:
/// - returns `Err(ExecErr)`
pub fn mollusk_exec(
    svm: &Mollusk,
    ixs: &[Instruction],
    accs_bef: &AccountMap,
) -> Result<ExecOk, ExecErr> {
    let accs_vec: Vec<_> = ixs
        .iter()
        .flat_map(|ix| ix.accounts.iter().map(|a| a.pubkey))
        .map(|k| {
            let (k, v) = accs_bef.get_key_value(&k).unwrap();
            (*k, v.clone())
        })
        .collect();

    let res = svm.process_instruction_chain(ixs, &accs_vec);

    match res.program_result {
        ProgramResult::Success => {
            let resulting_accounts: AccountMap = res.resulting_accounts.iter().cloned().collect();

            assert_balanced(accs_bef, &resulting_accounts);
            assert!(
                res.run_checks(&[Check::all_rent_exempt()], &svm.config, svm),
                "Not all accounts are rent-exempt after execution"
            );

            Ok(ExecOk {
                resulting_accounts,
                compute_units_consumed: res.compute_units_consumed,
                execution_time: res.execution_time,
                return_data: res.return_data,
            })
        }
        ProgramResult::Failure(e) => Err(ExecErr::Failure(e)),
        ProgramResult::UnknownError(e) => Err(ExecErr::UnknownError(e)),
    }
}

/// Returns `[bef, aft]`.
///
/// # Params
/// - `bef` should be the `accs_bef` passed to `mollusk_exec`
/// - `aft` should be `mollusk_exec(...).0`
pub fn acc_bef_aft<'a>(pk: &Pubkey, bef: &'a AccountMap, aft: &'a AccountMap) -> [&'a Account; 2] {
    [bef, aft].map(|m| m.get(pk).unwrap())
}

pub fn assert_jiminy_prog_err<E: Into<JiminyProgramError>>(exec_err: &ExecErr, expected: E) {
    match exec_err {
        ExecErr::Failure(actual) => {
            assert_prog_err_eq(actual, &expected.into());
        }
        err => {
            panic!("Expected Failure but got: {err:#?}");
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
