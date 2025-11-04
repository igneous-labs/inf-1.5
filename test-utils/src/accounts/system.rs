use solana_account::Account;

pub fn mock_sys_acc(lamports: u64) -> Account {
    Account {
        lamports,
        rent_epoch: u64::MAX,
        ..Default::default()
    }
}
