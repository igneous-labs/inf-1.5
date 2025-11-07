use std::{
    array,
    collections::{HashMap, HashSet},
    fs::File,
    path::{Path, PathBuf},
};

use glob::glob;
use inf1_ctl_core::keys::{
    DISABLE_POOL_AUTHORITY_LIST_ID, LST_STATE_LIST_ID, POOL_STATE_ID, REBALANCE_RECORD_ID,
    TOKEN_2022_ID,
};
use inf1_svc_lido_core::solido_legacy_core::SYSVAR_CLOCK;
use lazy_static::lazy_static;
use proptest::prelude::*;
use sanctum_marinade_liquid_staking_core::{SYSVAR_RENT, SYSVAR_STAKE_HISTORY};
use sanctum_spl_stake_pool_core::SYSVAR_STAKE_CONFIG;
use serde::{Deserialize, Serialize};
use solana_account::Account;
use solana_account_decoder_client_types::UiAccount;
use solana_pubkey::Pubkey;
use solido_legacy_core::TOKENKEG_PROGRAM;

use crate::{mock_clock, mock_prog_acc, mock_progdata_acc, PkAccountTup};

pub const JUPSOL_FIXTURE_LST_IDX: usize = 3;
pub const MSOL_FIXTURE_LST_IDX: usize = 2;

/// Programs that get built by `cargo-build-sbf` in the workspace
pub const LOCAL_PROGRAMS: [(&str, [u8; 32]); 2] = [
    ("inf1_pp_flatslab_program", inf1_pp_flatslab_core::ID),
    ("inf1_ctl_program", inf1_ctl_core::ID),
];

pub const FIXTURE_PROGRAMS: [(&str, [u8; 32]); 7] = [
    ("flat-fee-pp", inf1_pp_flatfee_core::ID),
    ("inf", inf1_ctl_core::ID),
    ("lido-calc", inf1_svc_lido_core::ID),
    ("marinade-calc", inf1_svc_marinade_core::ID),
    (
        "sanctum-spl-multi-calc",
        inf1_svc_spl_core::keys::sanctum_spl_multi::ID,
    ),
    ("spl-calc", inf1_svc_spl_core::keys::spl::ID),
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
                mollusk_svm::program::keyed_account_for_system_program(),
            ])
            .collect()
    };
}

/// Continues if fixture account not found for given pubkey
pub fn fixtures_accounts_opt_cloned(
    itr: impl IntoIterator<Item = impl Into<Pubkey>>,
) -> impl Iterator<Item = PkAccountTup> {
    itr.into_iter().filter_map(|pk| {
        let (k, v) = ALL_FIXTURES.get_key_value(&pk.into())?;
        Some((*k, v.clone()))
    })
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

lazy_static! {
    pub static ref RESERVED_PKS: HashSet<[u8; 32]> = [
        [0u8; 32],
        SYSVAR_CLOCK,
        SYSVAR_RENT,
        SYSVAR_STAKE_CONFIG,
        SYSVAR_STAKE_HISTORY,
        TOKENKEG_PROGRAM,
        TOKEN_2022_ID,
        POOL_STATE_ID,
        LST_STATE_LIST_ID,
        DISABLE_POOL_AUTHORITY_LIST_ID,
        REBALANCE_RECORD_ID,
    ]
    .into_iter()
    .collect();
}

/// Excludes:
/// - sysvars
/// - system program, token programs
/// - fixtures accounts
/// - controller program const PDAs thats supposed to contain data (everything except PROTOCOL_FEE_ID)
pub fn any_normal_pk() -> impl Strategy<Value = [u8; 32]> {
    any::<[u8; 32]>().prop_filter("not a normal pk", |pk| {
        !ALL_FIXTURES.contains_key(&Pubkey::new_from_array(*pk)) && !RESERVED_PKS.contains(pk)
    })
}

pub fn n_distinct_normal_pks<const N: usize>() -> impl Strategy<Value = [[u8; 32]; N]> {
    let end = array::from_fn(|_| Just([0u8; 32])).boxed();
    (0..N).fold(end, |end, i| {
        end.prop_flat_map(|end| {
            (
                // end defaults to system program, which is to be excluded anyway
                // so doesnt matter if we're comparing against uninitialized default data
                any_normal_pk().prop_filter("", move |pk| !end.contains(pk)),
                Just(end),
            )
        })
        .prop_map(move |(new, mut end)| {
            end[i] = new;
            end
        })
        .boxed()
    })
}
