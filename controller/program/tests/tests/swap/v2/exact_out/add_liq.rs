use std::collections::HashMap;

use expect_test::expect;
use inf1_ctl_jiminy::{
    instructions::swap::v2::{exact_out::NewSwapExactOutV2IxPreAccsBuilder, IxPreAccs},
    svc::InfDummyCalcAccs,
};
use inf1_pp_ag_core::{PricingAg, PricingAgTy};
use inf1_std::quote::Quote;
use inf1_svc_ag_core::{instructions::SvcCalcAccsAg, SvcAg, SvcAgTy};
use inf1_test_utils::{
    flatslab_fixture_suf_accs, jupsol_fixture_svc_suf_accs, mollusk_exec, KeyedUiAccount,
    JUPSOL_FIXTURE_LST_IDX,
};
use mollusk_svm::result::{InstructionResult, ProgramResult};

use crate::{common::SVM, tests::swap::common::assert_correct_swap_exact_out};

use super::{add_prog_accs, to_ix, Accs, Args};

#[test]
fn swap_exact_out_v2_jupsol_add_liq_fixture() {
    let curr_epoch = 0;
    let curr_slot = 0;
    let amount = 10_000;
    let prefix_am = NewSwapExactOutV2IxPreAccsBuilder::start()
        .with_signer("jupsol-token-acc-owner")
        .with_pool_state("pool-state")
        .with_lst_state_list("lst-state-list")
        .with_inp_acc("jupsol-token-acc")
        .with_inp_mint("jupsol-mint")
        .with_inp_pool_reserves("jupsol-reserves")
        .with_out_acc("inf-token-acc")
        .with_out_mint("inf-mint")
        .with_out_pool_reserves("inf-mint")
        .with_inp_token_program("tokenkeg")
        .with_out_token_program("tokenkeg")
        .build()
        .0
        .map(|n| KeyedUiAccount::from_test_fixtures_json(n).into_keyed_account());
    let prefix_keys = IxPreAccs(prefix_am.each_ref().map(|(addr, _)| addr.to_bytes()));
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
    let mut bef = prefix_am.into_iter().chain(pp_am).chain(inp_am).collect();
    add_prog_accs(&mut bef, &accs);
    let args = Args {
        inp_lst_index: JUPSOL_FIXTURE_LST_IDX.try_into().unwrap(),
        out_lst_index: u32::MAX,
        limit: u64::MAX,
        amount,
        accs,
    };
    let ix = to_ix(&args);

    let (
        _,
        InstructionResult {
            program_result,
            resulting_accounts,
            ..
        },
    ) = SVM.with(|svm| mollusk_exec(svm, &ix, &bef));
    let aft: HashMap<_, _> = resulting_accounts.into_iter().collect();

    assert_eq!(program_result, ProgramResult::Success);
    let Quote { inp, out, fee, .. } =
        assert_correct_swap_exact_out(&bef, &aft, &args, curr_epoch, curr_slot);
    expect![[r#"
        (
            12049,
            10000,
            121,
        )
    "#]]
    .assert_debug_eq(&(inp, out, fee));
}
