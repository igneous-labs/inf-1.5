use solana_account::Account;
use solana_pubkey::Pubkey;

/// Clock with everything = 0
pub fn mock_clock() -> Account {
    Account {
        data: vec![0; 40],
        owner: Pubkey::from_str_const("Sysvar1111111111111111111111111111111111111"),
        executable: false,
        // dont-cares
        lamports: 1169280,
        rent_epoch: u64::MAX,
    }
}
