use std::ops::Neg;

use inf1_ctl_jiminy::{
    accounts::pool_state::{PoolStateV2, PoolStateV2Packed, PoolStateV2U64s},
    instructions::swap::v2::IxPreAccs,
    typedefs::{
        pool_sv::PoolSvLamports,
        snap::{Snap, SnapU64},
    },
};
use inf1_pp_ag_core::instructions::{PriceExactInAccsAg, PriceExactOutAccsAg};

use inf1_std::quote::{
    swap::{exact_in::quote_exact_in, exact_out::quote_exact_out},
    Quote,
};
use inf1_test_utils::{
    acc_bef_aft, assert_diffs_pool_state_v2, assert_token_acc_diffs, get_mint_supply,
    token_acc_bal_diff_changed, AccountMap, Diff, DiffsPoolStateV2,
};
use sanctum_spl_token_jiminy::sanctum_spl_token_core::state::account::RawTokenAccount;
use sanctum_u64_ratio::Ratio;
use solana_pubkey::Pubkey;

use crate::{
    common::assert_lp_solvent_invar,
    tests::swap::common::{derive_pp_exact_in, derive_pp_exact_out, derive_qa_hla},
};

use super::super::V2Args;

pub fn assert_correct_swap_exact_in_v2(
    bef: &AccountMap,
    aft: &AccountMap,
    args: &V2Args<PriceExactInAccsAg>,
    curr_epoch: u64,
    curr_slot: u64,
) -> Quote {
    let pricing = derive_pp_exact_in(bef, &args.accs);
    let ps_aft =
        PoolStateV2Packed::of_acc_data(&aft[&(*args.accs.ix_prefix.pool_state()).into()].data)
            .unwrap()
            .into_pool_state_v2();
    let (qa, aft_header_la) = derive_qa_hla(bef, args, curr_epoch, curr_slot, pricing);
    let quote = quote_exact_in(&qa).unwrap();
    if args.inp_lst_index == u32::MAX || args.out_lst_index == u32::MAX {
        let inf_mint = if args.inp_lst_index == u32::MAX {
            args.accs.ix_prefix.inp_mint()
        } else {
            args.accs.ix_prefix.out_mint()
        };
        let inf_supply_snap =
            Snap([bef, aft].map(|am| get_mint_supply(&am[&(*inf_mint).into()].data)));
        assert_swap_token_movements(bef, aft, &args.accs.ix_prefix, &quote);
        assert_pool_state_liq(&aft_header_la, &ps_aft, quote.fee);
        assert_rr_liq(&aft_header_la, &ps_aft, &inf_supply_snap);
    } else {
        assert_swap_token_movements(bef, aft, &args.accs.ix_prefix, &quote);
        assert_pool_state_swap(&aft_header_la, &ps_aft, quote.fee);
    }
    quote
}

pub fn assert_correct_swap_exact_out_v2(
    bef: &AccountMap,
    aft: &AccountMap,
    args: &V2Args<PriceExactOutAccsAg>,
    curr_epoch: u64,
    curr_slot: u64,
) -> Quote {
    let pricing = derive_pp_exact_out(bef, &args.accs);
    let ps_aft =
        PoolStateV2Packed::of_acc_data(&aft[&(*args.accs.ix_prefix.pool_state()).into()].data)
            .unwrap()
            .into_pool_state_v2();
    let (qa, aft_header_la) = derive_qa_hla(bef, args, curr_epoch, curr_slot, pricing);
    let quote = quote_exact_out(&qa).unwrap();
    if args.inp_lst_index == u32::MAX || args.out_lst_index == u32::MAX {
        let inf_mint = if args.inp_lst_index == u32::MAX {
            args.accs.ix_prefix.inp_mint()
        } else {
            args.accs.ix_prefix.out_mint()
        };
        let inf_supply_snap =
            Snap([bef, aft].map(|am| get_mint_supply(&am[&(*inf_mint).into()].data)));
        assert_swap_token_movements(bef, aft, &args.accs.ix_prefix, &quote);
        assert_pool_state_liq(&aft_header_la, &ps_aft, quote.fee);
        assert_rr_liq(&aft_header_la, &ps_aft, &inf_supply_snap);
    } else {
        assert_swap_token_movements(bef, aft, &args.accs.ix_prefix, &quote);
        assert_pool_state_swap(&aft_header_la, &ps_aft, quote.fee);
    }
    quote
}

fn assert_swap_token_movements(
    bef: &AccountMap,
    aft: &AccountMap,
    accs: &IxPreAccs<impl Into<Pubkey> + Copy>,
    quote: &Quote,
) {
    let Quote {
        inp,
        out,
        inp_mint,
        out_mint,
        ..
    } = quote;

    // user's token accs
    let [user_inp, user_out] = [accs.inp_acc(), accs.out_acc()].map(|addr| {
        acc_bef_aft(&(*addr).into(), bef, aft)
            .map(|a| RawTokenAccount::of_acc_data(&a.data).unwrap())
    });
    [
        (user_inp, i128::from(*inp).neg()),
        (user_out, i128::from(*out)),
    ]
    .into_iter()
    .for_each(|([a_bef, a_aft], change)| {
        assert_token_acc_diffs(a_bef, a_aft, &token_acc_bal_diff_changed(a_bef, change))
    });

    let lp_mint =
        PoolStateV2Packed::of_acc_data(&aft.get(&(*accs.pool_state()).into()).unwrap().data)
            .unwrap()
            .into_pool_state_v2()
            .lp_token_mint;

    if *inp_mint == lp_mint {
        assert_pool_token_movements_rem_liq(bef, aft, accs, quote);
    } else if *out_mint == lp_mint {
        assert_pool_token_movements_add_liq(bef, aft, accs, quote);
    } else {
        assert_pool_token_movements_swap(bef, aft, accs, quote);
    }
}

fn assert_pool_token_movements_swap(
    bef: &AccountMap,
    aft: &AccountMap,
    accs: &IxPreAccs<impl Into<Pubkey> + Copy>,
    Quote { inp, out, .. }: &Quote,
) {
    let [inp_reserves, out_reserves] =
        [accs.inp_pool_reserves(), accs.out_pool_reserves()].map(|addr| {
            acc_bef_aft(&(*addr).into(), bef, aft)
                .map(|a| RawTokenAccount::of_acc_data(&a.data).unwrap())
        });
    [
        (inp_reserves, i128::from(*inp)),
        (out_reserves, i128::from(*out).neg()),
    ]
    .into_iter()
    .for_each(|([a_bef, a_aft], change)| {
        assert_token_acc_diffs(a_bef, a_aft, &token_acc_bal_diff_changed(a_bef, change))
    });
}

fn assert_pool_state_swap(aft_header_lookahead: &PoolStateV2, aft: &PoolStateV2, fee: u64) {
    let diffs = DiffsPoolStateV2 {
        u64s: PoolStateV2U64s::default()
            // checks below
            .with_total_sol_value(Diff::Pass)
            .with_withheld_lamports(Diff::Pass),
        ..Default::default()
    };
    let tsv_inc = aft.total_sol_value - aft_header_lookahead.total_sol_value;

    // might be > due to rounding?
    assert!(tsv_inc >= fee);

    let withheld_inc = aft.withheld_lamports - aft_header_lookahead.withheld_lamports;
    assert_eq!(withheld_inc, tsv_inc);

    assert_diffs_pool_state_v2(&diffs, aft_header_lookahead, aft);
    assert_lp_solvent_invar(aft);
}

fn assert_pool_token_movements_add_liq(
    bef: &AccountMap,
    aft: &AccountMap,
    accs: &IxPreAccs<impl Into<Pubkey> + Copy>,
    Quote { inp, out, .. }: &Quote,
) {
    let [inp_reserves_bef, inp_reserves_aft] =
        acc_bef_aft(&(*accs.inp_pool_reserves()).into(), bef, aft)
            .map(|a| RawTokenAccount::of_acc_data(&a.data).unwrap());
    assert_token_acc_diffs(
        inp_reserves_bef,
        inp_reserves_aft,
        &token_acc_bal_diff_changed(inp_reserves_bef, i128::from(*inp)),
    );
    let [lp_supp_bef, lp_supp_aft] =
        acc_bef_aft(&(*accs.out_mint()).into(), bef, aft).map(|a| get_mint_supply(&a.data));
    Diff::Changed(lp_supp_bef, lp_supp_bef + out).assert(&lp_supp_bef, &lp_supp_aft);
}

fn assert_pool_token_movements_rem_liq(
    bef: &AccountMap,
    aft: &AccountMap,
    accs: &IxPreAccs<impl Into<Pubkey> + Copy>,
    Quote { inp, out, .. }: &Quote,
) {
    let [lp_supp_bef, lp_supp_aft] =
        acc_bef_aft(&(*accs.inp_mint()).into(), bef, aft).map(|a| get_mint_supply(&a.data));
    Diff::Changed(lp_supp_bef, lp_supp_bef - inp).assert(&lp_supp_bef, &lp_supp_aft);
    let [out_reserves_bef, out_reserves_aft] =
        acc_bef_aft(&(*accs.out_pool_reserves()).into(), bef, aft)
            .map(|a| RawTokenAccount::of_acc_data(&a.data).unwrap());
    assert_token_acc_diffs(
        out_reserves_bef,
        out_reserves_aft,
        &token_acc_bal_diff_changed(out_reserves_bef, i128::from(*out).neg()),
    );
}

fn assert_pool_state_liq(aft_header_lookahead: &PoolStateV2, aft: &PoolStateV2, fee: u64) {
    let diffs = DiffsPoolStateV2 {
        u64s: PoolStateV2U64s::default()
            .with_withheld_lamports(Diff::Pass)
            // inc if add liq, dec if rem liq
            .with_total_sol_value(Diff::Pass),
        ..Default::default()
    };

    let withheld_inc = aft.withheld_lamports - aft_header_lookahead.withheld_lamports;
    assert_eq!(withheld_inc, fee);

    assert_diffs_pool_state_v2(&diffs, aft_header_lookahead, aft);
    assert_lp_solvent_invar(aft);
}

/// assert redemption rate of INF did not decrease after add/remove liq
fn assert_rr_liq(aft_header_lookahead: &PoolStateV2, aft: &PoolStateV2, inf_supply: &SnapU64) {
    let [bef_svl, aft_svl] = [aft_header_lookahead, aft].map(PoolSvLamports::from_pool_state_v2);

    let [[bef_total_ratio, bef_lp_ratio], [aft_total_ratio, aft_lp_ratio]] =
        [(bef_svl, inf_supply.old()), (aft_svl, inf_supply.new())].map(|(sv, sup)| {
            [
                Ratio {
                    n: *sv.total(),
                    d: *sup,
                },
                Ratio {
                    n: sv.lp_due_checked().unwrap(),
                    d: *sup,
                },
            ]
        });

    // should be checked by prog
    assert!(
        aft_total_ratio >= bef_total_ratio,
        "{bef_total_ratio:?}, {aft_total_ratio:?}"
    );

    // May be off by 1-2 due to rounding
    // TODO: assert this error bound
    let aft_lp_lenient = Ratio {
        n: aft_lp_ratio.n.saturating_add(2),
        d: aft_lp_ratio.d,
    };

    assert!(
        aft_lp_lenient >= bef_lp_ratio,
        "{bef_lp_ratio:?}, {aft_lp_lenient:?}"
    );
}
