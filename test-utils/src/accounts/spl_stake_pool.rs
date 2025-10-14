use sanctum_spl_stake_pool_core::StakePool;
use solana_account::Account;
use solana_pubkey::Pubkey;

/// Owner should be 1 of the 3 stake pool programs
pub fn mock_spl_stake_pool(a: StakePool, owner: Pubkey) -> Account {
    let mut data = Vec::new();
    a.borsh_ser(&mut data).unwrap();
    Account {
        lamports: 5_143_440, // solana rent 611
        data,
        owner,
        executable: false,
        rent_epoch: u64::MAX,
    }
}
