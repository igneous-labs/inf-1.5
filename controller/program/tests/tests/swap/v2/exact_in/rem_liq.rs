use expect_test::expect;
use inf1_ctl_jiminy::{
    instructions::swap::v2::{exact_out::NewSwapExactOutV2IxPreAccsBuilder, IxPreAccs},
    svc::InfDummyCalcAccs,
};
use inf1_pp_ag_core::{PricingAg, PricingAgTy};
use inf1_std::quote::Quote;
use inf1_svc_ag_core::{instructions::SvcCalcAccsAg, SvcAg, SvcAgTy};
use inf1_test_utils::{
    flatslab_fixture_suf_accs, jupsol_fixture_svc_suf_accs, mollusk_with_clock_override,
    silence_mollusk_logs, ClockArgs, ClockU64s, KeyedUiAccount, JUPSOL_FIXTURE_LST_IDX,
};
use jiminy_cpi::program_error::ProgramError;
use proptest::prelude::*;

use crate::{
    common::{SVM, SVM_MUT},
    tests::swap::{
        common::{
            assert_post_rem_all_liq, fill_swap_prog_accs, wsol_rem_liq_to_zero_inf_exact_in_strat,
        },
        V2Accs, V2Args,
    },
};

use super::swap_exact_in_v2_test;

#[test]
fn swap_exact_in_v2_jupsol_rem_liq_fixture() {
    let amount = 10_000;
    let prefix_am = IxPreAccs(
        NewSwapExactOutV2IxPreAccsBuilder::start()
            .with_signer("inf-token-acc-owner")
            .with_pool_state("pool-state")
            .with_lst_state_list("lst-state-list")
            .with_inp_acc("inf-token-acc")
            .with_inp_mint("inf-mint")
            .with_inp_pool_reserves("inf-mint")
            .with_out_acc("jupsol-token-acc")
            .with_out_mint("jupsol-mint")
            .with_out_pool_reserves("jupsol-reserves")
            // filler
            .with_inp_token_program("jupsol-mint")
            .with_out_token_program("jupsol-mint")
            .build()
            .0
            .map(|n| KeyedUiAccount::from_test_fixtures_json(n).into_keyed_account()),
    )
    .with_inp_token_program(mollusk_svm_programs_token::token::keyed_account())
    .with_out_token_program(mollusk_svm_programs_token::token::keyed_account());
    let prefix_keys = IxPreAccs(prefix_am.0.each_ref().map(|(addr, _)| addr.to_bytes()));
    let (pp_accs, pp_am) = flatslab_fixture_suf_accs();
    let (out_accs, out_am) = jupsol_fixture_svc_suf_accs();

    let accs = V2Accs {
        ix_prefix: prefix_keys,
        inp_calc_prog: inf1_ctl_jiminy::ID,
        inp_calc: SvcCalcAccsAg::Inf(InfDummyCalcAccs),
        out_calc_prog: *SvcAgTy::SanctumSplMulti(()).svc_program_id(),
        out_calc: SvcAg::SanctumSplMulti(out_accs),
        pricing_prog: *PricingAgTy::FlatSlab(()).program_id(),
        pricing: PricingAg::FlatSlab(pp_accs),
    };
    let args = V2Args {
        inp_lst_index: u32::MAX,
        out_lst_index: JUPSOL_FIXTURE_LST_IDX.try_into().unwrap(),
        limit: 0,
        amount,
        accs,
    };

    let mut bef = prefix_am.0.into_iter().chain(pp_am).chain(out_am).collect();
    fill_swap_prog_accs(&mut bef, &accs);

    let (Quote { inp, out, fee, .. }, _) =
        SVM.with(|svm| swap_exact_in_v2_test(svm, &args, &bef, None::<ProgramError>).unwrap());

    expect![[r#"
        (
            10000,
            19877,
            157,
        )
    "#]]
    .assert_debug_eq(&(inp, out, fee));
}

proptest! {
    #[test]
    fn swap_exact_in_v2_wsol_rem_to_zero_lp_supply(
        (slot, args, bef) in wsol_rem_liq_to_zero_inf_exact_in_strat()
    ) {
        silence_mollusk_logs();

        let (_, aft) = SVM_MUT.with_borrow_mut(
            |svm| mollusk_with_clock_override(
                svm,
                &ClockArgs {
                    u64s: ClockU64s::default().with_slot(Some(slot)),
                    ..Default::default()
                },
                |svm| swap_exact_in_v2_test(svm, &args, &bef, None::<ProgramError>).unwrap(),
            )
        );

        assert_post_rem_all_liq(&aft);
    }
}
