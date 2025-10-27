use solana_account::Account;
use solana_pubkey::Pubkey;

pub const BPF_LOADER_UPGRADEABLE_ADDR: Pubkey =
    Pubkey::from_str_const("BPFLoaderUpgradeab1e11111111111111111111111");

/// Creates a mock program account with given `programdata_address`
pub fn mock_prog_acc(programdata_address: Pubkey) -> Account {
    let mut data = vec![0u8; 36];
    // UpgradeableLoaderState::Program discriminant, is bincode enum
    data[0] = 2;
    data[4..].copy_from_slice(programdata_address.as_array());
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
pub fn mock_progdata_acc() -> Account {
    let mut data = vec![0u8; 45];
    // UpgradeableLoaderState::ProgramData discriminant, is bincode enum
    data[0] = 3;
    Account {
        data,
        owner: BPF_LOADER_UPGRADEABLE_ADDR,
        executable: false,
        // dont-cares
        lamports: 1_000_000_000,
        rent_epoch: u64::MAX,
    }
}
