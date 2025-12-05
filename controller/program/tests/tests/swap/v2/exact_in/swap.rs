use expect_test::expect;
use inf1_ctl_jiminy::instructions::swap::v2::IxPreAccs;
use inf1_pp_ag_core::{PricingAg, PricingAgTy};
use inf1_std::quote::Quote;
use inf1_svc_ag_core::{
    inf1_svc_wsol_core::instructions::sol_val_calc::WsolCalcAccs, instructions::SvcCalcAccsAg,
    SvcAg, SvcAgTy,
};
use inf1_test_utils::{
    flatslab_fixture_suf_accs, jupsol_fixture_svc_suf_accs, JUPSOL_FIXTURE_LST_IDX,
    WSOL_FIXTURE_LST_IDX,
};
use jiminy_cpi::program_error::ProgramError;

use crate::{
    common::SVM,
    tests::swap::{common::fill_swap_prog_accs, v2::jupsol_to_wsol_prefix_fixtures},
};

use super::{swap_exact_in_v2_test, Accs, Args};

#[test]
fn swap_exact_in_v2_jupsol_to_wsol_fixture() {
    let amount = 9_031;
    let prefix_am = jupsol_to_wsol_prefix_fixtures();
    let prefix_keys = IxPreAccs(prefix_am.0.each_ref().map(|(addr, _)| addr.to_bytes()));
    let out_accs = SvcCalcAccsAg::Wsol(WsolCalcAccs);
    let (pp_accs, pp_am) = flatslab_fixture_suf_accs();
    let (inp_accs, inp_am) = jupsol_fixture_svc_suf_accs();

    let accs = Accs {
        ix_prefix: prefix_keys,
        inp_calc_prog: *SvcAgTy::SanctumSplMulti(()).svc_program_id(),
        inp_calc: SvcAg::SanctumSplMulti(inp_accs),
        out_calc_prog: *SvcAgTy::Wsol(()).svc_program_id(),
        out_calc: out_accs,
        pricing_prog: *PricingAgTy::FlatSlab(()).program_id(),
        pricing: PricingAg::FlatSlab(pp_accs),
    };
    let args = Args {
        inp_lst_index: JUPSOL_FIXTURE_LST_IDX.try_into().unwrap(),
        out_lst_index: WSOL_FIXTURE_LST_IDX.try_into().unwrap(),
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
            9031,
            10002,
            51,
        )
    "#]]
    .assert_debug_eq(&(inp, out, fee));
}
