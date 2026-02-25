use inf1_ctl_core::{
    accounts::pool_state::{PoolStateV2FtaVals, VerPoolState},
    typedefs::versioned::V1_2,
};
use proptest::{prelude::Strategy, strategy::Union};

use crate::{
    any_pool_state, any_pool_state_v2, gen_pool_state, AnyPoolStateArgs, GenPoolStateArgs,
    PoolStateV2FtaStrat,
};

pub type VerPSAccArgs = V1_2<GenPoolStateArgs, PoolStateV2FtaVals>;

pub fn ver_pool_state_from_args(args: VerPSAccArgs) -> VerPoolState {
    match args {
        V1_2::V1(a) => V1_2::V1(gen_pool_state(a)),
        V1_2::V2(a) => V1_2::V2(a.into_pool_state_v2()),
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
