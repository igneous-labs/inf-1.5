use expect_test::expect;
use inf1_ctl_jiminy::{
    instructions::swap::v2::{exact_out::NewSwapExactOutV2IxPreAccsBuilder, IxPreAccs},
    svc::InfDummyCalcAccs,
};
use inf1_pp_ag_core::{PricingAg, PricingAgTy};
use inf1_std::quote::Quote;
use inf1_svc_ag_core::{instructions::SvcCalcAccsAg, SvcAg, SvcAgTy};
use inf1_test_utils::{
    flatslab_fixture_suf_accs, jupsol_fixture_svc_suf_accs, KeyedUiAccount, JUPSOL_FIXTURE_LST_IDX,
};
use jiminy_cpi::program_error::ProgramError;

use crate::{
    common::SVM,
    tests::swap::{common::fill_swap_prog_accs, V2Accs, V2Args},
};

use super::swap_exact_out_v2_test;

#[test]
fn swap_exact_out_v2_jupsol_add_liq_fixture() {
    let amount = 4_950;
    let prefix_am = IxPreAccs(
        NewSwapExactOutV2IxPreAccsBuilder::start()
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

    let accs = V2Accs {
        ix_prefix: prefix_keys,
        inp_calc_prog: *SvcAgTy::SanctumSplMulti(()).svc_program_id(),
        inp_calc: SvcAg::SanctumSplMulti(inp_accs),
        out_calc_prog: inf1_ctl_jiminy::ID,
        out_calc: SvcCalcAccsAg::Inf(InfDummyCalcAccs),
        pricing_prog: *PricingAgTy::FlatSlab(()).program_id(),
        pricing: PricingAg::FlatSlab(pp_accs),
    };
    let mut bef = prefix_am.0.into_iter().chain(pp_am).chain(inp_am).collect();
    fill_swap_prog_accs(&mut bef, &accs);
    let args = V2Args {
        inp_lst_index: JUPSOL_FIXTURE_LST_IDX.try_into().unwrap(),
        out_lst_index: u32::MAX,
        limit: u64::MAX,
        amount,
        accs,
    };

    let Quote { inp, out, fee, .. } =
        SVM.with(|svm| swap_exact_out_v2_test(svm, &args, &bef, None::<ProgramError>).unwrap());

    expect![[r#"
        (
            10003,
            4950,
            101,
        )
    "#]]
    .assert_debug_eq(&(inp, out, fee));
}
