use jiminy_sysvar_rent::Rent;
use solana_account::Account;
use solana_pubkey::Pubkey;

pub fn slab_account(slab_data: Vec<u8>) -> Account {
    let lamports = Rent::DEFAULT.min_balance(slab_data.len());
    Account {
        data: slab_data,
        owner: Pubkey::new_from_array(inf1_pp_flatslab_core::ID),
        lamports,
        executable: false,
        rent_epoch: u64::MAX,
    }
}
