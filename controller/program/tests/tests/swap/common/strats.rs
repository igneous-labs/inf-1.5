use inf1_ctl_jiminy::{
    accounts::{
        lst_state_list::LstStatePackedList,
        pool_state::{PoolStateV2, PoolStateV2Addrs},
    },
    svc::InfDummyCalcAccs,
    typedefs::pool_sv::PoolSvMutRefs,
};
use inf1_pp_ag_core::{PricingAg, PricingAgTy};
use inf1_pp_core::pair::Pair;
use inf1_svc_ag_core::{
    inf1_svc_wsol_core::{self, instructions::sol_val_calc::WsolCalcAccs},
    instructions::SvcCalcAccsAg,
    SvcAg, SvcAgTy,
};
use inf1_test_utils::{
    any_lst_state, any_lst_state_list, any_pool_state_v2, bals_from_supply, n_distinct_normal_pks,
    pool_state_v2_u64s_with_last_release_slot_bef_incl, pool_state_v2_u8_bools_normal_strat,
    pool_sv_lamports_solvent_strat, reasonable_flatslab_strat_for_mints, AccountMap,
    AnyLstStateArgs, LstStateListData, LstStatePks, NewLstStatePksBuilder, PoolStateV2FtaStrat,
    VerPS, WSOL_MINT,
};
use proptest::prelude::*;

use crate::tests::swap::{
    common::{
        fill_swap_prog_accs, swap_pre_accs, NewSwapTokenAddrsBuilder, NewSwapTokenU64sBuilder,
        SwapTokenArg,
    },
    V2Accs, V2Args,
};

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

/// Returns `(curr_slot, args, account_map)`
pub fn add_liq_wsol_zero_inf_strat() -> impl Strategy<Value = (u64, V2Args, AccountMap)> {
    let sol_val_and_inp_amt = bals_from_supply::<2>(u64::MAX).prop_map(|(bals, _)| bals);

    (any::<u64>(), sol_val_and_inp_amt)
        .prop_flat_map(|(curr_slot, [sol_val, inp_amt])| {
            (
                n_distinct_normal_pks(),
                swap_prog_accs_strat(
                    [AnyLstStateArgs {
                        pks: wsol_lst_state_pks(),
                        sol_value: Some(Just(sol_val).boxed()),
                        is_input_disabled: Some(Just(false).boxed()),
                        ..Default::default()
                    }],
                    PoolStateV2FtaStrat {
                        u64s: pool_state_v2_u64s_with_last_release_slot_bef_incl(
                            Default::default(),
                            curr_slot,
                        ),
                        u8_bools: pool_state_v2_u8_bools_normal_strat(),
                        addrs: PoolStateV2Addrs::default().with_pricing_program(Some(
                            Just(*PricingAgTy::FlatSlab(()).program_id()).boxed(),
                        )),
                        ..Default::default()
                    },
                )
                .prop_flat_map(|([idx], lsl, ps)| {
                    (
                        reasonable_flatslab_strat_for_mints(
                            [ps.lp_token_mint, WSOL_MINT.to_bytes()]
                                .into_iter()
                                .collect(),
                        ),
                        Just((idx, lsl, ps)),
                    )
                }),
                Just(curr_slot),
                Just((sol_val, inp_amt)),
            )
        })
        .prop_map(
            |(
                [signer, inp_acc, out_acc],
                ((pp_accs, pp_am), (idx, lsl, ps)),
                curr_slot,
                (wsol_sol_val, inp_amt),
            )| {
                let (ix_prefix, ix_prefix_am) = swap_pre_accs(
                    &signer,
                    &VerPS::V2(ps),
                    &lsl,
                    &Pair {
                        inp: SwapTokenArg {
                            u64s: NewSwapTokenU64sBuilder::start()
                                .with_acc_bal(inp_amt)
                                .with_mint_supply(u64::MAX)
                                .with_reserves_bal(wsol_sol_val)
                                .build(),
                            addrs: NewSwapTokenAddrsBuilder::start()
                                .with_acc(inp_acc)
                                .with_mint(WSOL_MINT.to_bytes())
                                .build(),
                        },
                        out: SwapTokenArg {
                            u64s: NewSwapTokenU64sBuilder::start()
                                .with_acc_bal(0)
                                // always 0 LP mint supply
                                .with_mint_supply(0)
                                .with_reserves_bal(0)
                                .build(),
                            addrs: NewSwapTokenAddrsBuilder::start()
                                .with_acc(out_acc)
                                .with_mint(ps.lp_token_mint)
                                .build(),
                        },
                    },
                );

                let accs = V2Accs {
                    ix_prefix,
                    inp_calc_prog: *SvcAgTy::Wsol(()).svc_program_id(),
                    inp_calc: SvcAg::Wsol(WsolCalcAccs),
                    out_calc_prog: inf1_ctl_jiminy::ID,
                    out_calc: SvcCalcAccsAg::Inf(InfDummyCalcAccs),
                    pricing_prog: *PricingAgTy::FlatSlab(()).program_id(),
                    pricing: PricingAg::FlatSlab(pp_accs),
                };
                let args = V2Args {
                    inp_lst_index: idx.try_into().unwrap(),
                    out_lst_index: u32::MAX,
                    limit: 0,
                    amount: inp_amt,
                    accs,
                };

                let mut bef = ix_prefix_am.into_iter().chain(pp_am).collect();
                fill_swap_prog_accs(&mut bef, &accs);

                (curr_slot, args, bef)
            },
        )
}
