use std::{
    collections::HashMap,
    fs::File,
    path::{Path, PathBuf},
};

use glob::glob;
use inf1_svc_lido_core::solido_legacy_core::SYSVAR_CLOCK;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use solana_account::Account;
use solana_account_decoder_client_types::UiAccount;
use solana_pubkey::Pubkey;

use crate::{mock_clock, mock_prog_acc, mock_progdata_acc};

/// Programs that get built by `cargo-build-sbf` in the workspace
pub const LOCAL_PROGRAMS: [(&str, [u8; 32]); 2] = [
    ("inf1_pp_flatslab_program", inf1_pp_flatslab_core::ID),
    ("inf1_ctl_program", inf1_ctl_core::ID),
];

pub const FIXTURE_PROGRAMS: [(&str, [u8; 32]); 6] = [
    ("flat-fee-pp", inf1_pp_flatfee_core::ID),
    ("inf", inf1_ctl_core::ID),
    ("lido-calc", inf1_svc_lido_core::ID),
    ("marinade-calc", inf1_svc_marinade_core::ID),
    (
        "sanctum-spl-multi-calc",
        inf1_svc_spl_core::keys::sanctum_spl_multi::ID,
    ),
    ("wsol-calc", inf1_svc_wsol_core::ID),
];

lazy_static! {
    pub static ref ALL_FIXTURES: HashMap<Pubkey, Account> = {
        let abs_json_paths = glob(test_fixtures_dir().join("*.json").to_str().unwrap()).unwrap();
        abs_json_paths
            .map(|p| KeyedUiAccount::from_file(p.unwrap()).into_keyed_account())
            .chain(
                [
                    (
                        inf1_svc_spl_core::keys::sanctum_spl_multi::POOL_PROG_ID,
                        inf1_svc_spl_core::keys::sanctum_spl_multi::POOL_PROGDATA_ID,
                    ),
                    (
                        inf1_svc_lido_core::keys::POOL_PROG_ID,
                        inf1_svc_lido_core::keys::POOL_PROGDATA_ID,
                    ),
                    (
                        inf1_svc_marinade_core::keys::POOL_PROG_ID,
                        inf1_svc_marinade_core::keys::POOL_PROGDATA_ID,
                    ),
                ]
                .into_iter()
                .flat_map(|(prog_id, prog_data_id)| {
                    let prog_data_id = Pubkey::new_from_array(prog_data_id);
                    [
                        (Pubkey::new_from_array(prog_id), mock_prog_acc(prog_data_id)),
                        (prog_data_id, mock_progdata_acc()),
                    ]
                }),
            )
            .chain(LOCAL_PROGRAMS.into_iter().chain(FIXTURE_PROGRAMS).map(|(_, prog_id)| {
                (
                    Pubkey::new_from_array(prog_id),
                    // dont-care, doesnt affect mollusk, program is added to ProgramCache
                    // via other mechanism
                    mock_prog_acc(Default::default()),
                )
            }))
            .chain([
                (SYSVAR_CLOCK.into(), mock_clock()),
                mollusk_svm_programs_token::token::keyed_account(),
                mollusk_svm_programs_token::associated_token::keyed_account(),
            ])
            .collect()
    };
}

/// Copied from https://stackoverflow.com/a/74942075/5057425
pub fn workspace_root_dir() -> PathBuf {
    let output = std::process::Command::new(env!("CARGO"))
        .arg("locate-project")
        .arg("--workspace")
        .arg("--message-format=plain")
        .output()
        .unwrap()
        .stdout;
    let cargo_path = Path::new(std::str::from_utf8(&output).unwrap().trim());
    cargo_path.parent().unwrap().to_path_buf()
}

/// Returns `/path/to/workspace/root/test-fixtures`
pub fn test_fixtures_dir() -> PathBuf {
    workspace_root_dir().join("test-fixtures")
}

/// This is the json format of
/// `solana account -o <FILENAME>.json --output json <ACCOUNT-PUBKEY>`
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KeyedUiAccount {
    pub pubkey: String,
    pub account: UiAccount,
}

impl KeyedUiAccount {
    pub fn from_file<P: AsRef<Path>>(json_file_path: P) -> Self {
        let mut file = File::open(json_file_path).unwrap();
        serde_json::from_reader(&mut file).unwrap()
    }

    pub fn from_test_fixtures_json(p: &str) -> Self {
        Self::from_file(test_fixtures_dir().join(p).with_extension("json"))
    }

    pub fn into_keyed_account(self) -> (Pubkey, Account) {
        let Self { pubkey, account } = self;
        (pubkey.parse().unwrap(), account.decode().unwrap())
    }
}
