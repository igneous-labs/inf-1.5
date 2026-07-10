use inf1_svc_ag_core::inf1_svc_generic::accounts::state::State;

use crate::Diff;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DiffsGenSvcState {
    pub manager: Diff<[u8; 32]>,
    pub last_upgrade_slot: Diff<u64>,
}

pub fn assert_diffs_gen_svc_state(diffs: &DiffsGenSvcState, bef: &State, aft: &State) {
    diffs.manager.assert(&bef.manager, &aft.manager);
    diffs
        .last_upgrade_slot
        .assert(&bef.last_upgrade_slot, &aft.last_upgrade_slot);
}
