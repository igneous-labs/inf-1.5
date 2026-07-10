use inf1_svc_ag_core::inf1_svc_generic::accounts::state::State;
use proptest::prelude::*;

use crate::{pk_strat, u64_strat};

pub fn any_gen_svc_state() -> impl Strategy<Value = State> {
    (pk_strat(None), u64_strat(None)).prop_map(|(manager, last_upgrade_slot)| State {
        manager,
        last_upgrade_slot,
    })
}
