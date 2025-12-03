use inf1_ctl_core::{
    accounts::lst_state_list::LstStatePackedList, keys::SYS_PROG_ID, typedefs::lst_state::LstState,
};
use jiminy_sysvar_rent::Rent;
use solana_account::Account;
use solana_pubkey::Pubkey;

pub fn lst_state_list_account(data: Vec<u8>) -> Account {
    let (lamports, owner) = if data.is_empty() {
        // Empty account owned by system program
        (0, Pubkey::new_from_array(SYS_PROG_ID))
    } else {
        (
            Rent::DEFAULT.min_balance(data.len()),
            Pubkey::new_from_array(inf1_ctl_core::ID),
        )
    };

    Account {
        lamports,
        data,
        owner,
        executable: false,
        rent_epoch: u64::MAX,
    }
}

pub fn get_lst_state_list(lst_state_list_data: &[u8]) -> Vec<LstState> {
    LstStatePackedList::of_acc_data(lst_state_list_data)
        .unwrap()
        .0
        .iter()
        .map(|s| s.into_lst_state())
        .collect()
}
