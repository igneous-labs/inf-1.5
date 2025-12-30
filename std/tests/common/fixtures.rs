use inf1_std::inf1_ctl_core::{
    accounts::{
        lst_state_list::LstStatePackedList,
        pool_state::{PoolState, PoolStatePacked},
    },
    keys::{LST_STATE_LIST_ID, POOL_STATE_ID},
    typedefs::lst_state::LstState,
};
use inf1_test_utils::ALL_FIXTURES;

pub fn pool_state_fixture() -> PoolState {
    PoolStatePacked::of_acc_data(&ALL_FIXTURES[&POOL_STATE_ID.into()].data)
        .unwrap()
        .into_pool_state()
}

pub fn lst_state_list_fixture() -> Vec<LstState> {
    LstStatePackedList::of_acc_data(&ALL_FIXTURES[&LST_STATE_LIST_ID.into()].data)
        .unwrap()
        .0
        .iter()
        .map(|l| l.into_lst_state())
        .collect()
}
