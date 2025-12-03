use std::ops::Neg;

use inf1_ctl_jiminy::{
    accounts::pool_state::{PoolStateV2, PoolStateV2Packed, PoolStateV2U64s},
    instructions::swap::v2::IxPreAccs,
    svc::InfCalc,
    typedefs::{
        pool_sv::PoolSvLamports,
        snap::{Snap, SnapU64},
    },
};
use inf1_pp_ag_core::{instructions::PriceExactOutAccsAg, PricingAg};
use inf1_pp_core::pair::Pair;
use inf1_pp_flatslab_std::{accounts::Slab, pricing::FlatSlabSwapPricing};
use inf1_std::quote::{
    swap::{exact_out::quote_exact_out, QuoteArgs},
    Quote,
};
use inf1_svc_ag_core::{
    calc::SvcCalcAg,
    inf1_svc_spl_core::{
        calc::SplCalc,
        instructions::sol_val_calc::{SanctumSplCalcAccs, SanctumSplMultiCalcAccs, SplCalcAccs},
        sanctum_spl_stake_pool_core::StakePool,
    },
    inf1_svc_wsol_core::calc::WsolCalc,
    instructions::SvcCalcAccsAg,
};
use inf1_test_utils::{
    acc_bef_aft, assert_diffs_pool_state_v2, assert_token_acc_diffs, fill_mock_prog_accs,
    get_lst_state_list, get_mint_supply, get_token_account_amount, token_acc_bal_diff_changed,
    AccountMap, Diff, DiffsPoolStateV2, VerPoolState,
};
use sanctum_spl_token_jiminy::sanctum_spl_token_core::state::account::RawTokenAccount;
use sanctum_u64_ratio::Ratio;
use solana_pubkey::Pubkey;

use crate::common::{assert_lp_solvent_invar, header_lookahead, Cbs};

use super::{Accs, Args};

pub fn add_swap_prog_accs<P>(
    am: &mut AccountMap,
    Accs {
        inp_calc_prog,
        out_calc_prog,
        pricing_prog,
        ..
    }: &Accs<P>,
) {
    fill_mock_prog_accs(am, [*inp_calc_prog, *out_calc_prog, *pricing_prog]);
}

pub fn assert_correct_swap_exact_out(
    bef: &AccountMap,
    aft: &AccountMap,
    args: &Args<PriceExactOutAccsAg>,
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
    if args.inp_lst_index == u32::MAX {
        // rem liq
        assert_swap_token_movements(bef, aft, &args.accs.ix_prefix, &quote);
        assert_pool_state_liq(&aft_header_la, &ps_aft, quote.fee);
        let inf_supply_snap = Snap(
            [bef, aft]
                .map(|am| get_mint_supply(&am[&(*args.accs.ix_prefix.inp_mint()).into()].data)),
        );
        assert_rr_liq(&aft_header_la, &ps_aft, &inf_supply_snap);
    } else if args.out_lst_index == u32::MAX {
        // add liq
        assert_swap_token_movements(bef, aft, &args.accs.ix_prefix, &quote);
        assert_pool_state_liq(&aft_header_la, &ps_aft, quote.fee);
        let inf_supply_snap = Snap(
            [bef, aft]
                .map(|am| get_mint_supply(&am[&(*args.accs.ix_prefix.out_mint()).into()].data)),
        );
        assert_rr_liq(&aft_header_la, &ps_aft, &inf_supply_snap);
    } else {
        assert_swap_token_movements(bef, aft, &args.accs.ix_prefix, &quote);
        assert_pool_state_swap(&aft_header_la, &ps_aft, quote.fee);
    }
    quote
}

/// Derive quote args and header lookahead
fn derive_qa_hla<P, T>(
    am: &AccountMap,
    args: &Args<T>,
    curr_epoch: u64,
    curr_slot: u64,
    // passthrough to generalize
    // across both ExactIn and ExactOut
    pricing: P,
) -> (QuoteArgs<SvcCalcAg, SvcCalcAg, P>, PoolStateV2) {
    let ((inp_calc, out_calc, aft_header_la), out_reserves) = if args.inp_lst_index == u32::MAX {
        (
            derive_rem_liq_cahla(am, args, curr_epoch, curr_slot),
            get_token_account_amount(&am[&(*args.accs.ix_prefix.out_pool_reserves()).into()].data),
        )
    } else if args.out_lst_index == u32::MAX {
        (
            derive_add_liq_cahla(am, args, curr_epoch, curr_slot),
            u64::MAX,
        )
    } else {
        (
            derive_swap_cahla(am, args, curr_epoch, curr_slot),
            get_token_account_amount(&am[&(*args.accs.ix_prefix.out_pool_reserves()).into()].data),
        )
    };
    (
        QuoteArgs {
            amt: args.amount,
            out_reserves,
            inp_mint: *args.accs.ix_prefix.inp_mint(),
            out_mint: *args.accs.ix_prefix.out_mint(),
            inp_calc,
            out_calc,
            pricing,
        },
        aft_header_la,
    )
}

/// `_cahla` - `calcs and header lookahead`
/// Returns (inp_calc, out_calc, ps_header_lookahead)
fn derive_swap_cahla<P>(
    am: &AccountMap,
    args: &Args<P>,
    curr_epoch: u64,
    curr_slot: u64,
) -> (SvcCalcAg, SvcCalcAg, PoolStateV2) {
    let [inp_calc, out_calc] =
        [args.accs.inp_calc, args.accs.out_calc].map(|c| derive_svc_no_inf(am, &c, curr_epoch));
    let [inp_reserves_bal, out_reserves_bal] = [
        args.accs.ix_prefix.inp_pool_reserves(),
        args.accs.ix_prefix.out_pool_reserves(),
    ]
    .map(|a| get_token_account_amount(&am[&(*a).into()].data));
    let ps = ps_header_lookahead(
        am,
        &args.accs.ix_prefix,
        &[
            (&inp_calc, inp_reserves_bal, args.inp_lst_index as usize),
            (&out_calc, out_reserves_bal, args.out_lst_index as usize),
        ],
        curr_slot,
    );
    (inp_calc, out_calc, ps)
}

fn derive_add_liq_cahla<P>(
    am: &AccountMap,
    args: &Args<P>,
    curr_epoch: u64,
    curr_slot: u64,
) -> (SvcCalcAg, SvcCalcAg, PoolStateV2) {
    let inp_calc = derive_svc_no_inf(am, &args.accs.inp_calc, curr_epoch);
    let inp_reserves_balance =
        get_token_account_amount(&am[&(*args.accs.ix_prefix.inp_pool_reserves()).into()].data);
    let inf_mint_supply = get_mint_supply(&am[&(*args.accs.ix_prefix.out_mint()).into()].data);
    let ps = ps_header_lookahead(
        am,
        &args.accs.ix_prefix,
        &[(&inp_calc, inp_reserves_balance, args.inp_lst_index as usize)],
        curr_slot,
    );
    (
        inp_calc,
        SvcCalcAg::Inf(InfCalc::new(&ps, inf_mint_supply)),
        ps,
    )
}

fn derive_rem_liq_cahla<P>(
    am: &AccountMap,
    args: &Args<P>,
    curr_epoch: u64,
    curr_slot: u64,
) -> (SvcCalcAg, SvcCalcAg, PoolStateV2) {
    let out_calc = derive_svc_no_inf(am, &args.accs.out_calc, curr_epoch);
    let out_reserves_bal =
        get_token_account_amount(&am[&(*args.accs.ix_prefix.out_pool_reserves()).into()].data);
    let inf_mint_supply = get_mint_supply(&am[&(*args.accs.ix_prefix.inp_mint()).into()].data);
    let ps = ps_header_lookahead(
        am,
        &args.accs.ix_prefix,
        &[(&out_calc, out_reserves_bal, args.out_lst_index as usize)],
        curr_slot,
    );
    (
        SvcCalcAg::Inf(InfCalc::new(&ps, inf_mint_supply)),
        out_calc,
        ps,
    )
}

fn ps_header_lookahead(
    am: &AccountMap,
    ix_prefix: &IxPreAccs<impl Into<Pubkey> + Copy>,
    calcs: &[(&SvcCalcAg, u64, usize)],
    curr_slot: u64,
) -> PoolStateV2 {
    let ps = VerPoolState::from_acc_data(&am[&(*ix_prefix.pool_state()).into()].data)
        .migrated(curr_slot);
    let lst_state_list = get_lst_state_list(&am[&(*ix_prefix.lst_state_list()).into()].data);
    let calcs = calcs.iter().map(|(calc, balance, idx)| Cbs {
        calc,
        balance: *balance,
        old_sol_val: lst_state_list[*idx].sol_value,
    });
    header_lookahead(ps, calcs, curr_slot)
}

fn derive_svc_no_inf(am: &AccountMap, accs: &SvcCalcAccsAg, curr_epoch: u64) -> SvcCalcAg {
    match accs {
        SvcCalcAccsAg::Wsol(_) => SvcCalcAg::Wsol(WsolCalc),
        SvcCalcAccsAg::SanctumSplMulti(SanctumSplMultiCalcAccs { stake_pool_addr })
        | SvcCalcAccsAg::SanctumSpl(SanctumSplCalcAccs { stake_pool_addr })
        | SvcCalcAccsAg::Spl(SplCalcAccs { stake_pool_addr }) => {
            let calc = SplCalc::new(
                &StakePool::borsh_de(am[&(*stake_pool_addr).into()].data.as_slice()).unwrap(),
                curr_epoch,
            );
            match accs {
                SvcCalcAccsAg::SanctumSplMulti(_) => SvcCalcAg::SanctumSplMulti(calc),
                SvcCalcAccsAg::SanctumSpl(_) => SvcCalcAg::SanctumSpl(calc),
                SvcCalcAccsAg::Spl(_) => SvcCalcAg::Spl(calc),
                _ => unreachable!(),
            }
        }
        SvcCalcAccsAg::Inf(_) => panic!("INF unsupported"),
        _ => todo!(),
    }
}

fn derive_pp_exact_out(am: &AccountMap, accs: &Accs<PriceExactOutAccsAg>) -> FlatSlabSwapPricing {
    match accs.pricing {
        PricingAg::FlatSlab(p) => Slab::of_acc_data(&am[&(*p.0.slab()).into()].data)
            .unwrap()
            .entries()
            .pricing(&Pair {
                inp: accs.ix_prefix.inp_mint(),
                out: accs.ix_prefix.out_mint(),
            })
            .unwrap(),
        PricingAg::FlatFee(_) => todo!(),
    }
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
