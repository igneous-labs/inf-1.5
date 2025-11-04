use solana_account::Account;

pub fn mock_system_acc(lamports: u64) -> Account {
    Account {
        lamports,
        rent_epoch: u64::MAX,
        ..Default::default()
    }
}
