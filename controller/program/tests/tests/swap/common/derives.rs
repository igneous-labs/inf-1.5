use inf1_ctl_jiminy::{
    accounts::pool_state::{PoolStateV2, PoolStateV2Packed},
    instructions::swap::v2::IxPreAccs,
    svc::InfCalc,
    typedefs::lst_state::LstState,
};
use inf1_pp_ag_core::PricingAg;
use inf1_pp_core::pair::Pair;
use inf1_pp_flatslab_std::{
    accounts::Slab, instructions::pricing::FlatSlabPpAccs, pricing::FlatSlabSwapPricing,
};
use inf1_std::quote::swap::QuoteArgs;
use inf1_svc_ag_core::calc::SvcCalcAg;
use inf1_test_utils::{
    get_lst_state_list, get_mint_supply, get_token_account_amount, AccountMap, VerPoolState,
};
use solana_pubkey::Pubkey;

use crate::{
    common::{derive_svc_no_inf, header_lookahead, lst_state_lookahead, Cbs},
    tests::swap::{PricingSwapAg, QuoteArgsAg},
};

use super::super::{V2Accs, V2Args};

pub fn derive_qa_prog_accs(
    bef: &AccountMap,
    aft: &AccountMap,
    args: &V2Args,
    curr_epoch: u64,
    curr_slot: u64,
) -> ([PoolStateV2; 2], [Vec<LstState>; 2], QuoteArgsAg) {
    let ps_aft =
        PoolStateV2Packed::of_acc_data(&aft[&(*args.accs.ix_prefix.pool_state()).into()].data)
            .unwrap()
            .into_pool_state_v2();
    let list_aft = get_lst_state_list(&aft[&(*args.accs.ix_prefix.lst_state_list()).into()].data);
    let (qa, ps_aft_header_la, list_aft_header_la) =
        derive_qa_hla(bef, args, curr_epoch, curr_slot);
    (
        [ps_aft_header_la, ps_aft],
        [list_aft_header_la, list_aft],
        qa,
    )
}

pub fn derive_qa_hla(
    am: &AccountMap,
    args: &V2Args,
    curr_epoch: u64,
    curr_slot: u64,
) -> (QuoteArgsAg, PoolStateV2, Vec<LstState>) {
    let pricing = derive_pp(am, &args.accs);
    let ((inp_calc, out_calc, ps_aft_header_la, list_aft_header_la), out_reserves) = if args
        .inp_lst_index
        == u32::MAX
    {
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
        ps_aft_header_la,
        list_aft_header_la,
    )
}

/// `_cahla` - `calcs and header lookahead`
/// Returns (inp_calc, out_calc, ps_header_lookahead)
fn derive_swap_cahla(
    am: &AccountMap,
    args: &V2Args,
    curr_epoch: u64,
    curr_slot: u64,
) -> (SvcCalcAg, SvcCalcAg, PoolStateV2, Vec<LstState>) {
    let [inp_calc, out_calc] =
        [args.accs.inp_calc, args.accs.out_calc].map(|c| derive_svc_no_inf(am, &c, curr_epoch));
    let [inp_reserves_bal, out_reserves_bal] = [
        args.accs.ix_prefix.inp_pool_reserves(),
        args.accs.ix_prefix.out_pool_reserves(),
    ]
    .map(|a| get_token_account_amount(&am[&(*a).into()].data));
    let [inp_lst_index, out_lst_index] =
        [args.inp_lst_index, args.out_lst_index].map(|x| x as usize);

    let params = [
        (&inp_calc, inp_reserves_bal, inp_lst_index),
        (&out_calc, out_reserves_bal, out_lst_index),
    ];
    let ps = ps_header_lookahead(am, &args.accs.ix_prefix, &params, curr_slot);

    let mut list = get_lst_state_list(&am[&(*args.accs.ix_prefix.lst_state_list()).into()].data);
    params.into_iter().for_each(|(calc, bal, idx)| {
        list[idx] = lst_state_lookahead(list[idx], bal, calc);
    });

    (inp_calc, out_calc, ps, list)
}

fn derive_add_liq_cahla(
    am: &AccountMap,
    args: &V2Args,
    curr_epoch: u64,
    curr_slot: u64,
) -> (SvcCalcAg, SvcCalcAg, PoolStateV2, Vec<LstState>) {
    let inp_calc = derive_svc_no_inf(am, &args.accs.inp_calc, curr_epoch);
    let inp_reserves_balance =
        get_token_account_amount(&am[&(*args.accs.ix_prefix.inp_pool_reserves()).into()].data);
    let inf_mint_supply = get_mint_supply(&am[&(*args.accs.ix_prefix.out_mint()).into()].data);
    let idx = args.inp_lst_index as usize;
    let ps = ps_header_lookahead(
        am,
        &args.accs.ix_prefix,
        &[(&inp_calc, inp_reserves_balance, idx)],
        curr_slot,
    );

    let mut list = get_lst_state_list(&am[&(*args.accs.ix_prefix.lst_state_list()).into()].data);
    list[idx] = lst_state_lookahead(list[idx], inp_reserves_balance, inp_calc);

    (
        inp_calc,
        SvcCalcAg::Inf(InfCalc::new(&ps, inf_mint_supply)),
        ps,
        list,
    )
}

fn derive_rem_liq_cahla(
    am: &AccountMap,
    args: &V2Args,
    curr_epoch: u64,
    curr_slot: u64,
) -> (SvcCalcAg, SvcCalcAg, PoolStateV2, Vec<LstState>) {
    let out_calc = derive_svc_no_inf(am, &args.accs.out_calc, curr_epoch);
    let out_reserves_bal =
        get_token_account_amount(&am[&(*args.accs.ix_prefix.out_pool_reserves()).into()].data);
    let inf_mint_supply = get_mint_supply(&am[&(*args.accs.ix_prefix.inp_mint()).into()].data);
    let idx = args.out_lst_index as usize;

    let ps = ps_header_lookahead(
        am,
        &args.accs.ix_prefix,
        &[(&out_calc, out_reserves_bal, idx)],
        curr_slot,
    );

    let mut list = get_lst_state_list(&am[&(*args.accs.ix_prefix.lst_state_list()).into()].data);
    list[idx] = lst_state_lookahead(list[idx], out_reserves_bal, out_calc);

    (
        SvcCalcAg::Inf(InfCalc::new(&ps, inf_mint_supply)),
        out_calc,
        ps,
        list,
    )
}

// TODO: these utils may be generally useful for other tests like
// SyncSolVal and Rebalance

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

pub fn derive_pp(am: &AccountMap, accs: &V2Accs) -> PricingSwapAg {
    match accs.pricing {
        PricingAg::FlatSlab(p) => PricingAg::FlatSlab(flatslab_pricing(am, accs, &p)),
        PricingAg::FlatFee(_) => todo!(),
    }
}

fn flatslab_pricing(am: &AccountMap, accs: &V2Accs, p: &FlatSlabPpAccs) -> FlatSlabSwapPricing {
    Slab::of_acc_data(&am[&(*p.0.slab()).into()].data)
        .unwrap()
        .entries()
        .pricing(&Pair {
            inp: accs.ix_prefix.inp_mint(),
            out: accs.ix_prefix.out_mint(),
        })
        .unwrap()
}
