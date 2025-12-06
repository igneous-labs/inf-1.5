use inf1_ctl_jiminy::{
    accounts::{lst_state_list::LstStatePackedList, pool_state::PoolStateV2},
    typedefs::pool_sv::PoolSvMutRefs,
};
use inf1_svc_ag_core::inf1_svc_wsol_core;
use inf1_test_utils::{
    any_lst_state, any_lst_state_list, any_pool_state_v2, pool_sv_lamports_solvent_strat,
    AnyLstStateArgs, LstStateListData, LstStatePks, NewLstStatePksBuilder, PoolStateV2FtaStrat,
    WSOL_MINT,
};
use proptest::prelude::*;

pub fn wsol_lst_state_pks() -> LstStatePks<Option<BoxedStrategy<[u8; 32]>>> {
    LstStatePks(
        NewLstStatePksBuilder::start()
            .with_mint(WSOL_MINT.to_bytes())
            .with_sol_value_calculator(inf1_svc_wsol_core::ID)
            .build()
            .0
            .map(|x| Some(Just(x).boxed())),
    )
}

/// ps_args.u64s is ignored for pool sv lamport fields
/// in order to maintain invariant that sum of lst_state_list.sol_value
/// = pool_state.total_sol_value
///
/// Currently does not include other entries in lst_state_list to not have to
/// deal with sum of total sol values overflowing u64
pub fn swap_prog_accs_strat<const N: usize>(
    lst_args: [AnyLstStateArgs; N],
    ps_args: PoolStateV2FtaStrat,
) -> impl Strategy<Value = ([usize; N], LstStateListData, PoolStateV2)> {
    (
        lst_args.map(|a| any_lst_state(a, None)),
        any_lst_state_list(Default::default(), None, 0..=0),
        any_pool_state_v2(ps_args),
    )
        .prop_flat_map(|(lsds, mut lsl, ps)| {
            let idxs = lsds.map(|lsd| lsl.upsert(lsd));

            let tsv = LstStatePackedList::of_acc_data(&lsl.lst_state_list)
                .unwrap()
                .0
                .iter()
                .map(|s| s.into_lst_state().sol_value)
                .sum::<u64>();

            (
                pool_sv_lamports_solvent_strat(tsv),
                Just(idxs),
                Just(lsl),
                Just(ps),
            )
        })
        .prop_map(move |(psv, idxs, lsl, mut ps)| {
            PoolSvMutRefs::from_pool_state_v2(&mut ps).update(psv);
            (idxs, lsl, ps)
        })
}
