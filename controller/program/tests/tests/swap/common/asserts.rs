use std::ops::Neg;

use inf1_ctl_jiminy::{
    accounts::pool_state::{PoolStateV2, PoolStateV2Packed, PoolStateV2U64s},
    instructions::swap::v2::IxPreAccs,
    typedefs::{
        lst_state::LstState,
        pool_sv::PoolSvLamports,
        snap::{Snap, SnapU64},
    },
};

use inf1_std::quote::{
    swap::{exact_in::quote_exact_in, exact_out::quote_exact_out},
    Quote,
};
use inf1_test_utils::{
    acc_bef_aft, assert_diffs_lst_state_list, assert_diffs_pool_state_v2, assert_token_acc_diffs,
    get_mint_supply, token_acc_bal_diff_changed, AccountMap, Diff, DiffsPoolStateV2,
    LstStateListChanges,
};
use sanctum_spl_token_jiminy::sanctum_spl_token_core::state::account::RawTokenAccount;
use sanctum_u64_ratio::Ratio;
use solana_pubkey::Pubkey;

use crate::{common::assert_lp_solvent_invar, tests::swap::common::derive_qa_prog_accs};

use super::super::V2Args;

pub fn assert_correct_swap_exact_in_v2(
    bef: &AccountMap,
    aft: &AccountMap,
    args: &V2Args,
    curr_epoch: u64,
    curr_slot: u64,
) -> Quote {
    let (ps, list, qa) = derive_qa_prog_accs(bef, aft, args, curr_epoch, curr_slot);
    let quote = quote_exact_in(&qa).unwrap();
    assert_correct_swap_v2(
        bef,
        aft,
        args,
        ps.each_ref(),
        list.each_ref().map(AsRef::as_ref),
        &quote,
    );
    // slippage limit should have been respected
    assert!(quote.out >= args.limit);
    quote
}

pub fn assert_correct_swap_exact_out_v2(
    bef: &AccountMap,
    aft: &AccountMap,
    args: &V2Args,
    curr_epoch: u64,
    curr_slot: u64,
) -> Quote {
    let (ps, list, qa) = derive_qa_prog_accs(bef, aft, args, curr_epoch, curr_slot);
    let quote = quote_exact_out(&qa).unwrap();
    assert_correct_swap_v2(
        bef,
        aft,
        args,
        ps.each_ref(),
        list.each_ref().map(AsRef::as_ref),
        &quote,
    );
    // slippage limit should have been respected
    assert!(quote.inp <= args.limit);
    quote
}

fn assert_correct_swap_v2(
    bef: &AccountMap,
    aft: &AccountMap,
    args: &V2Args,
    ps: [&PoolStateV2; 2],
    list: [&[LstState]; 2],
    quote: &Quote,
) {
    if args.inp_lst_index == u32::MAX || args.out_lst_index == u32::MAX {
        let inf_mint = if args.inp_lst_index == u32::MAX {
            args.accs.ix_prefix.inp_mint()
        } else {
            args.accs.ix_prefix.out_mint()
        };
        let inf_supply_snap =
            Snap([bef, aft].map(|am| get_mint_supply(&am[&(*inf_mint).into()].data)));
        assert_swap_token_movements(bef, aft, &args.accs.ix_prefix, quote);
        assert_accs_liq(ps, list, quote);
        assert_rr_liq(ps, &inf_supply_snap);
    } else {
        assert_swap_token_movements(bef, aft, &args.accs.ix_prefix, quote);
        assert_accs_swap(ps, list, quote);
    }
}

/// Assert that tokens have moved according to `quote`
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

fn assert_accs_swap(
    [ps_aft_hla, ps_aft]: [&PoolStateV2; 2],
    [list_aft_hla, list_aft]: [&[LstState]; 2],
    Quote {
        fee,
        inp_mint,
        out_mint,
        ..
    }: &Quote,
) {
    // TODO: verify this error bound and verify that its due to rounding.
    // Pool's sol val inc is allowed to be greater than the quoted fee by at most this much
    const FEE_ERR_BOUND_LAMPORTS: u64 = 3;

    let ps_diffs = DiffsPoolStateV2 {
        u64s: PoolStateV2U64s::default()
            // checks below
            .with_total_sol_value(Diff::Pass)
            .with_withheld_lamports(Diff::Pass),
        ..Default::default()
    };
    let tsv_inc = ps_aft.total_sol_value - ps_aft_hla.total_sol_value;

    assert!(tsv_inc >= *fee);
    assert!(tsv_inc <= fee + FEE_ERR_BOUND_LAMPORTS);

    let withheld_inc = ps_aft.withheld_lamports - ps_aft_hla.withheld_lamports;
    assert_eq!(withheld_inc, tsv_inc);

    assert_diffs_pool_state_v2(&ps_diffs, ps_aft_hla, ps_aft);
    assert_lp_solvent_invar(ps_aft);

    let (list_diffs, inp_svc) =
        LstStateListChanges::new(list_aft_hla).with_det_svc_by_mint(inp_mint, list_aft);
    let (list_diffs, out_svc) = list_diffs.with_det_svc_by_mint(out_mint, list_aft);

    // assert everything else other than sol value didnt change
    assert_diffs_lst_state_list(list_diffs.build(), list_aft_hla, list_aft);

    assert!(inp_svc >= 0);
    assert!(out_svc <= 0);

    assert_eq!(
        inp_svc + out_svc,
        i128::from(tsv_inc),
        "{} - {} != {}",
        inp_svc,
        out_svc.neg(),
        tsv_inc
    );

    // sum sol value = total_sol_value invariant
    assert_eq!(
        list_aft.iter().map(|s| s.sol_value).sum::<u64>(),
        ps_aft.total_sol_value
    );
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

fn assert_accs_liq(
    [ps_aft_hla, ps_aft]: [&PoolStateV2; 2],
    [list_aft_hla, list_aft]: [&[LstState]; 2],
    Quote {
        fee,
        inp_mint,
        out_mint,
        ..
    }: &Quote,
) {
    let diffs = DiffsPoolStateV2 {
        u64s: PoolStateV2U64s::default()
            .with_withheld_lamports(Diff::Pass)
            // inc if add liq, dec if rem liq
            .with_total_sol_value(Diff::Pass),
        ..Default::default()
    };

    let tsv_change = i128::from(ps_aft.total_sol_value) - i128::from(ps_aft_hla.total_sol_value);

    let withheld_inc = ps_aft.withheld_lamports - ps_aft_hla.withheld_lamports;
    assert_eq!(withheld_inc, *fee);

    assert_diffs_pool_state_v2(&diffs, ps_aft_hla, ps_aft);
    assert_lp_solvent_invar(ps_aft);

    let list_diffs = LstStateListChanges::new(list_aft_hla);
    let (list_diffs, lst_svc) = if *inp_mint == ps_aft.lp_token_mint {
        list_diffs.with_det_svc_by_mint(out_mint, list_aft)
    } else {
        list_diffs.with_det_svc_by_mint(inp_mint, list_aft)
    };

    assert_eq!(lst_svc, tsv_change);

    // assert everything else other than sol value didnt change
    assert_diffs_lst_state_list(list_diffs.build(), list_aft_hla, list_aft);

    // sum sol value = total_sol_value invariant
    assert_eq!(
        list_aft.iter().map(|s| s.sol_value).sum::<u64>(),
        ps_aft.total_sol_value
    );
}

/// assert redemption rate of INF did not decrease after add/remove liq
fn assert_rr_liq([aft_header_lookahead, aft]: [&PoolStateV2; 2], inf_supply: &SnapU64) {
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
