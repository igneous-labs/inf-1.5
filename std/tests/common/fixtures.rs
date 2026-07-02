use inf1_std::inf1_ctl_core::{
    accounts::{lst_state_list::LstStatePackedList, pool_state::VerPoolState},
    pda::CONST_PDA_KEYS_OWNED,
    typedefs::lst_state::LstState,
};
use inf1_test_utils::ALL_FIXTURES;

pub fn pool_state_fixture() -> VerPoolState {
    VerPoolState::try_from_acc_data(
        &ALL_FIXTURES[&Into::into(*CONST_PDA_KEYS_OWNED.pool_state())].data,
    )
    .unwrap()
}

pub fn lst_state_list_fixture() -> Vec<LstState> {
    LstStatePackedList::of_acc_data(
        &ALL_FIXTURES[&Into::into(*CONST_PDA_KEYS_OWNED.lst_state_list())].data,
    )
    .unwrap()
    .0
    .iter()
    .map(|l| l.into_lst_state())
    .collect()
}
