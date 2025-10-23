use jiminy_sysvar_rent::Rent;
use sanctum_marinade_liquid_staking_core::SYSTEM_PROGRAM;
use solana_account::Account;
use solana_pubkey::Pubkey;

pub fn mock_system_acc(data: Vec<u8>) -> Account {
    Account {
        lamports: Rent::DEFAULT.min_balance(data.len()),
        data,
        owner: Pubkey::new_from_array(SYSTEM_PROGRAM),
        executable: false,
        rent_epoch: u64::MAX,
    }
}
