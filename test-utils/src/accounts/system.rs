use jiminy_sysvar_rent::Rent;
use solana_account::Account;

pub fn mock_system_acc(lamports: Option<u64>) -> Account {
    Account {
        lamports: match lamports {
            Some(l) => l,
            None => Rent::DEFAULT.min_balance(0),
        },
        ..Default::default()
    }
}
