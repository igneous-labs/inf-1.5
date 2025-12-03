use solana_account::Account;
use solana_pubkey::Pubkey;

use crate::AccountMap;

pub const BPF_LOADER_UPGRADEABLE_ADDR: Pubkey =
    Pubkey::from_str_const("BPFLoaderUpgradeab1e11111111111111111111111");

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProgramDataAddr {
    ProgAddr(Pubkey),
    Raw(Pubkey),
}

impl ProgramDataAddr {
    pub fn into_pda(self) -> Pubkey {
        match self {
            ProgramDataAddr::ProgAddr(pk) => {
                Pubkey::find_program_address(&[pk.as_ref()], &BPF_LOADER_UPGRADEABLE_ADDR).0
            }
            ProgramDataAddr::Raw(pk) => pk,
        }
    }
}

/// Creates a mock program account with given `programdata_address`,
/// or defaults to the PDA
pub fn mock_prog_acc(programdata_address: ProgramDataAddr) -> Account {
    let mut data = vec![0u8; 36];
    // UpgradeableLoaderState::Program discriminant, is bincode enum
    data[0] = 2;
    data[4..].copy_from_slice(programdata_address.into_pda().as_array());
    Account {
        data,
        owner: BPF_LOADER_UPGRADEABLE_ADDR,
        executable: true,
        // dont-cares
        lamports: 1_000_000_000,
        rent_epoch: u64::MAX,
    }
}

/// Creates a mock program data account with last upgrade slot 0
pub fn mock_progdata_acc(last_upgrade_slot: u64) -> Account {
    let mut data = vec![0u8; 45];
    // UpgradeableLoaderState::ProgramData discriminant, is bincode enum
    data[0] = 3;

    data[4..12].copy_from_slice(&last_upgrade_slot.to_le_bytes());

    Account {
        data,
        owner: BPF_LOADER_UPGRADEABLE_ADDR,
        executable: false,
        // dont-cares
        lamports: 1_000_000_000,
        rent_epoch: u64::MAX,
    }
}

/// Assumes we dont care abt progdata addr
pub fn fill_mock_prog_accs(
    am: &mut AccountMap,
    prog_addrs: impl IntoIterator<Item = impl Into<Pubkey>>,
) {
    am.extend(prog_addrs.into_iter().map(|addr| {
        (
            addr.into(),
            // dont-care
            mock_prog_acc(ProgramDataAddr::Raw(Default::default())),
        )
    }));
}
