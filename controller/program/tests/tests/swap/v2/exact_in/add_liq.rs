use expect_test::expect;
use inf1_ctl_jiminy::{
    accounts::pool_state::PoolStateV2Addrs,
    instructions::swap::v2::{exact_in::NewSwapExactInV2IxPreAccsBuilder, IxPreAccs},
    keys::{LST_STATE_LIST_ID, POOL_STATE_ID},
    svc::InfDummyCalcAccs,
};
use inf1_pp_ag_core::{PricingAg, PricingAgTy};
use inf1_std::quote::Quote;
use inf1_svc_ag_core::{
    inf1_svc_wsol_core::instructions::sol_val_calc::WsolCalcAccs, instructions::SvcCalcAccsAg,
    SvcAg, SvcAgTy,
};
use inf1_test_utils::{
    bals_from_supply, flatslab_fixture_suf_accs, jupsol_fixture_svc_suf_accs,
    lst_state_list_account, mock_mint, mock_sys_acc, mock_token_acc, mollusk_with_clock_override,
    n_distinct_normal_pks, pool_state_v2_account,
    pool_state_v2_u64s_with_last_release_slot_bef_incl, pool_state_v2_u8_bools_normal_strat,
    raw_mint, raw_token_acc, reasonable_flatslab_strat_for_mints, silence_mollusk_logs, AccountMap,
    AnyLstStateArgs, ClockArgs, ClockU64s, KeyedUiAccount, PoolStateV2FtaStrat,
    JUPSOL_FIXTURE_LST_IDX, WSOL_MINT,
};
use jiminy_cpi::program_error::ProgramError;
use proptest::prelude::*;
use solana_pubkey::Pubkey;

use crate::{
    common::{SVM, SVM_MUT},
    tests::swap::common::{fill_swap_prog_accs, swap_prog_accs_strat, wsol_lst_state_pks},
};

use super::{swap_exact_in_v2_test, Accs, Args};

#[test]
fn swap_exact_in_v2_jupsol_add_liq_fixture() {
    let amount = 10_000;
    let prefix_am = IxPreAccs(
        NewSwapExactInV2IxPreAccsBuilder::start()
            .with_signer("jupsol-token-acc-owner")
            .with_pool_state("pool-state")
            .with_lst_state_list("lst-state-list")
            .with_inp_acc("jupsol-token-acc")
            .with_inp_mint("jupsol-mint")
            .with_inp_pool_reserves("jupsol-reserves")
            .with_out_acc("inf-token-acc")
            .with_out_mint("inf-mint")
            .with_out_pool_reserves("inf-mint")
            // filler
            .with_inp_token_program("inf-mint")
            .with_out_token_program("inf-mint")
            .build()
            .0
            .map(|n| KeyedUiAccount::from_test_fixtures_json(n).into_keyed_account()),
    )
    .with_inp_token_program(mollusk_svm_programs_token::token::keyed_account())
    .with_out_token_program(mollusk_svm_programs_token::token::keyed_account());
    let prefix_keys = IxPreAccs(prefix_am.0.each_ref().map(|(addr, _)| addr.to_bytes()));
    let (pp_accs, pp_am) = flatslab_fixture_suf_accs();
    let (inp_accs, inp_am) = jupsol_fixture_svc_suf_accs();

    let accs = Accs {
        ix_prefix: prefix_keys,
        inp_calc_prog: *SvcAgTy::SanctumSplMulti(()).svc_program_id(),
        inp_calc: SvcAg::SanctumSplMulti(inp_accs),
        out_calc_prog: inf1_ctl_jiminy::ID,
        out_calc: SvcCalcAccsAg::Inf(InfDummyCalcAccs),
        pricing_prog: *PricingAgTy::FlatSlab(()).program_id(),
        pricing: PricingAg::FlatSlab(pp_accs),
    };
    let args = Args {
        inp_lst_index: JUPSOL_FIXTURE_LST_IDX.try_into().unwrap(),
        out_lst_index: u32::MAX,
        limit: 0,
        amount,
        accs,
    };

    let mut bef = prefix_am.0.into_iter().chain(pp_am).chain(inp_am).collect();
    fill_swap_prog_accs(&mut bef, &accs);

    let Quote { inp, out, fee, .. } =
        SVM.with(|svm| swap_exact_in_v2_test(svm, &args, &bef, None::<ProgramError>).unwrap());

    expect![[r#"
        (
            10000,
            4949,
            101,
        )
    "#]]
    .assert_debug_eq(&(inp, out, fee));
}

fn add_liq_wsol_zero_inf_strat() -> impl Strategy<Value = (u64, Args, AccountMap)> {
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
                let lp_mint = (
                    Pubkey::new_from_array(ps.lp_token_mint),
                    // always 0 supply
                    mock_mint(raw_mint(Some(POOL_STATE_ID), None, 0, 9)),
                );
                let accounts = NewSwapExactInV2IxPreAccsBuilder::start()
                    .with_signer((signer.into(), mock_sys_acc(1_000_000_000)))
                    .with_inp_acc((
                        inp_acc.into(),
                        mock_token_acc(raw_token_acc(WSOL_MINT.to_bytes(), signer, inp_amt)),
                    ))
                    .with_out_acc((
                        out_acc.into(),
                        mock_token_acc(raw_token_acc(ps.lp_token_mint, signer, 0)),
                    ))
                    .with_inp_mint((WSOL_MINT, mock_mint(raw_mint(None, None, u64::MAX, 9))))
                    .with_inp_pool_reserves((
                        lsl.all_pool_reserves[WSOL_MINT.as_array()].into(),
                        mock_token_acc(raw_token_acc(
                            WSOL_MINT.to_bytes(),
                            POOL_STATE_ID,
                            wsol_sol_val,
                        )),
                    ))
                    .with_inp_token_program(mollusk_svm_programs_token::token::keyed_account())
                    .with_out_mint(lp_mint.clone())
                    .with_out_pool_reserves(lp_mint)
                    .with_out_token_program(mollusk_svm_programs_token::token::keyed_account())
                    .with_pool_state((POOL_STATE_ID.into(), pool_state_v2_account(ps)))
                    .with_lst_state_list((
                        LST_STATE_LIST_ID.into(),
                        lst_state_list_account(lsl.lst_state_list),
                    ))
                    .build();
                let ix_prefix = IxPreAccs(accounts.0.each_ref().map(|(pk, _)| pk.to_bytes()));

                let accs = Accs {
                    ix_prefix,
                    inp_calc_prog: *SvcAgTy::Wsol(()).svc_program_id(),
                    inp_calc: SvcAg::Wsol(WsolCalcAccs),
                    out_calc_prog: inf1_ctl_jiminy::ID,
                    out_calc: SvcCalcAccsAg::Inf(InfDummyCalcAccs),
                    pricing_prog: *PricingAgTy::FlatSlab(()).program_id(),
                    pricing: PricingAg::FlatSlab(pp_accs),
                };
                let args = Args {
                    inp_lst_index: idx.try_into().unwrap(),
                    out_lst_index: u32::MAX,
                    limit: 0,
                    amount: inp_amt,
                    accs,
                };

                let mut bef = accounts.0.into_iter().chain(pp_am).collect();
                fill_swap_prog_accs(&mut bef, &accs);

                (curr_slot, args, bef)
            },
        )
}

proptest! {
    #[test]
    fn swap_exact_in_v2_wsol_add_from_zero_lp_supply(
        (slot, args, bef) in add_liq_wsol_zero_inf_strat()
    ) {
        silence_mollusk_logs();

        SVM_MUT.with_borrow_mut(
            |svm| mollusk_with_clock_override(
                svm,
                &ClockArgs {
                    u64s: ClockU64s::default().with_slot(Some(slot)),
                    ..Default::default()
                },
                |svm| swap_exact_in_v2_test(svm, &args, &bef, None::<ProgramError>).unwrap(),
            )
        );
    }
}
