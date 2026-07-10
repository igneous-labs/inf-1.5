use inf1_svc_ag_core::inf1_svc_generic::accounts::state::State;
use solana_account::Account;
use solana_pubkey::Pubkey;

pub fn mock_gen_svc_state(data: State, owner: Pubkey) -> Account {
    Account {
        lamports: 1_169_280, // solana rent 40
        data: data.as_acc_data_arr().into(),
        owner,
        executable: false,
        rent_epoch: u64::MAX,
    }
}
