use inf1_ctl_core::accounts::pool_state::PoolStateV2FtaVals;
use proptest::{prelude::Strategy, strategy::Union};

use crate::{
    any_pool_state, any_pool_state_v2, gen_pool_state, AnyPoolStateArgs, GenPoolStateArgs,
    PoolStateV2FtaStrat, VerPS, VerPoolState,
};

pub type VerPSAccArgs = VerPS<GenPoolStateArgs, PoolStateV2FtaVals>;

impl VerPoolState {
    pub fn from_args(args: VerPSAccArgs) -> Self {
        match args {
            VerPS::V1(a) => VerPS::V1(gen_pool_state(a)),
            VerPS::V2(a) => VerPS::V2(a.into_pool_state_v2()),
        }
    }
}

pub fn any_pool_state_ver(
    v1: AnyPoolStateArgs,
    v2: PoolStateV2FtaStrat,
) -> impl Strategy<Value = VerPoolState> {
    Union::new([
        any_pool_state(v1).prop_map(VerPoolState::V1).boxed(),
        any_pool_state_v2(v2).prop_map(VerPoolState::V2).boxed(),
    ])
}
