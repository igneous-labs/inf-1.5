use inf1_ctl_jiminy::{
    accounts::pool_state::PoolStateV2, instructions::swap::v2::IxPreAccs, svc::InfCalc,
};
use inf1_pp_ag_core::{
    instructions::{PriceExactInAccsAg, PriceExactOutAccsAg},
    PricingAg,
};
use inf1_pp_core::pair::Pair;
use inf1_pp_flatslab_std::{
    accounts::Slab, instructions::pricing::FlatSlabPpAccs, pricing::FlatSlabSwapPricing,
};
use inf1_std::quote::swap::QuoteArgs;
use inf1_svc_ag_core::{
    calc::SvcCalcAg,
    inf1_svc_lido_core::{calc::LidoCalc, solido_legacy_core},
    inf1_svc_marinade_core::{calc::MarinadeCalc, sanctum_marinade_liquid_staking_core},
    inf1_svc_spl_core::{
        calc::SplCalc,
        instructions::sol_val_calc::{SanctumSplCalcAccs, SanctumSplMultiCalcAccs, SplCalcAccs},
        sanctum_spl_stake_pool_core::StakePool,
    },
    inf1_svc_wsol_core::calc::WsolCalc,
    instructions::SvcCalcAccsAg,
};
use inf1_test_utils::{
    get_lst_state_list, get_mint_supply, get_token_account_amount, AccountMap, VerPoolState,
};
use solana_pubkey::Pubkey;

use crate::common::{header_lookahead, Cbs};

use super::super::{Accs, Args};

/// Derive quote args and header lookahead
pub fn derive_qa_hla<P, T>(
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
        SvcCalcAccsAg::Marinade(_) => SvcCalcAg::Marinade(MarinadeCalc::new(
            &sanctum_marinade_liquid_staking_core::State::borsh_de(
                am[&sanctum_marinade_liquid_staking_core::STATE_PUBKEY.into()]
                    .data
                    .as_slice(),
            )
            .unwrap(),
        )),
        SvcCalcAccsAg::Lido(_) => SvcCalcAg::Lido(LidoCalc::new(
            &solido_legacy_core::Lido::borsh_de(
                am[&solido_legacy_core::LIDO_STATE_ADDR.into()]
                    .data
                    .as_slice(),
            )
            .unwrap(),
            curr_epoch,
        )),
        SvcCalcAccsAg::Inf(_) => panic!("INF unsupported"),
    }
}

pub fn derive_pp_exact_in(am: &AccountMap, accs: &Accs<PriceExactInAccsAg>) -> FlatSlabSwapPricing {
    match accs.pricing {
        PricingAg::FlatSlab(p) => flatslab_pricing(am, accs, &p),
        PricingAg::FlatFee(_) => todo!(),
    }
}

pub fn derive_pp_exact_out(
    am: &AccountMap,
    accs: &Accs<PriceExactOutAccsAg>,
) -> FlatSlabSwapPricing {
    match accs.pricing {
        PricingAg::FlatSlab(p) => flatslab_pricing(am, accs, &p),
        PricingAg::FlatFee(_) => todo!(),
    }
}

fn flatslab_pricing(
    am: &AccountMap,
    accs: &Accs<PriceExactOutAccsAg>,
    p: &FlatSlabPpAccs,
) -> FlatSlabSwapPricing {
    Slab::of_acc_data(&am[&(*p.0.slab()).into()].data)
        .unwrap()
        .entries()
        .pricing(&Pair {
            inp: accs.ix_prefix.inp_mint(),
            out: accs.ix_prefix.out_mint(),
        })
        .unwrap()
}
