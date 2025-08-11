use std::{collections::HashMap, fs::File, path::Path};

use glob::glob;
use inf1_std::inf1_svc_ag_std::{
    inf1_svc_lido_core::{self, solido_legacy_core::SYSVAR_CLOCK},
    inf1_svc_marinade_core,
    inf1_svc_spl_core::keys::sanctum_spl_multi,
};
use jupiter_amm_interface::{AmmContext, ClockRef};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use solana_account::Account;
use solana_account_decoder_client_types::UiAccount;
use solana_pubkey::Pubkey;

use crate::common::INF_PROGRAMS;

use super::test_fixtures_dir;

pub const JUPSOL_MINT_ADDR: [u8; 32] =
    Pubkey::from_str_const("jupSoLaHXQiZZTSfEWMTRRgpnyFm8f6sZdosWBjx93v").to_bytes();

lazy_static! {
    pub static ref ALL_FIXTURES: HashMap<Pubkey, Account> = {
        let abs_json_paths = glob(test_fixtures_dir().join("*.json").to_str().unwrap()).unwrap();
        abs_json_paths
            .map(|p| KeyedUiAccount::from_file(p.unwrap()).into_keyed_account())
            .chain(
                [
                    (
                        sanctum_spl_multi::POOL_PROG_ID,
                        sanctum_spl_multi::POOL_PROGDATA_ID,
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
            .chain(INF_PROGRAMS.map(|(_, prog_id)| {
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
            ])
            .collect()
    };
    pub static ref AMM_CONTEXT: AmmContext = {
        AmmContext {
            clock_ref: ClockRef::default(),
        }
    };
}

/// This is the json format of
/// `solana account -o <FILENAME>.json --output json <ACCOUNT-PUBKEY>`
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KeyedUiAccount {
    pub pubkey: String,
    pub account: UiAccount,
}

impl KeyedUiAccount {
    fn from_file<P: AsRef<Path>>(json_file_path: P) -> Self {
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

/// Creates a mock program account with given `programdata_address`
fn mock_prog_acc(programdata_address: Pubkey) -> Account {
    let mut data = vec![0u8; 36];
    // UpgradeableLoaderState::Program discriminant, is bincode enum
    data[0] = 2;
    data[4..].copy_from_slice(programdata_address.as_array());
    Account {
        data,
        owner: Pubkey::from_str_const("BPFLoaderUpgradeab1e11111111111111111111111"),
        executable: false,
        // dont-cares
        lamports: 1_000_000_000,
        rent_epoch: u64::MAX,
    }
}

/// Creates a mock program data account with last upgrade slot 0
fn mock_progdata_acc() -> Account {
    let mut data = vec![0u8; 45];
    // UpgradeableLoaderState::ProgramData discriminant, is bincode enum
    data[0] = 3;
    Account {
        data,
        owner: Pubkey::from_str_const("BPFLoaderUpgradeab1e11111111111111111111111"),
        executable: false,
        // dont-cares
        lamports: 1_000_000_000,
        rent_epoch: u64::MAX,
    }
}

/// Clock with everything = 0
fn mock_clock() -> Account {
    Account {
        data: vec![0; 40],
        owner: Pubkey::from_str_const("Sysvar1111111111111111111111111111111111111"),
        executable: false,
        // dont-cares
        lamports: 1169280,
        rent_epoch: u64::MAX,
    }
}
