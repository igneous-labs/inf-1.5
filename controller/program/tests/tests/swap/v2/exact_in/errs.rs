use inf1_ctl_jiminy::{
    accounts::lst_state_list::LstStatePackedListMut,
    err::Inf1CtlErr,
    instructions::swap::v2::{exact_out::NewSwapExactOutV2IxPreAccsBuilder, IxPreAccs},
    program_err::Inf1CtlCustomProgErr,
    typedefs::u8bool::U8BoolMut,
};
use inf1_pp_ag_core::{PricingAg, PricingAgTy};
use inf1_svc_ag_core::{
    inf1_svc_wsol_core::instructions::sol_val_calc::WsolCalcAccs, instructions::SvcCalcAccsAg,
    SvcAg, SvcAgTy,
};
use inf1_test_utils::{
    flatslab_fixture_suf_accs, jupsol_fixture_svc_suf_accs, KeyedUiAccount, VerPoolState,
    JUPSOL_FIXTURE_LST_IDX, JUPSOL_MINT, WSOL_FIXTURE_LST_IDX,
};

use crate::{
    common::SVM,
    tests::swap::{
        common::fill_swap_prog_accs,
        v2::{
            exact_in::{swap_exact_in_v2_test, Accs, Args},
            jupsol_to_wsol_prefix_fixtures,
        },
    },
};

#[test]
fn swap_exact_in_input_disabled_fixture() {
    let mut prefix_am = jupsol_to_wsol_prefix_fixtures();
    let prefix_keys = IxPreAccs(prefix_am.0.each_ref().map(|(addr, _)| addr.to_bytes()));
    let out_accs = SvcCalcAccsAg::Wsol(WsolCalcAccs);
    let (pp_accs, pp_am) = flatslab_fixture_suf_accs();
    let (inp_accs, inp_am) = jupsol_fixture_svc_suf_accs();

    let lst_state_list =
        LstStatePackedListMut::of_acc_data(&mut prefix_am.lst_state_list_mut().1.data).unwrap();
    let lst_state = lst_state_list
        .0
        .iter_mut()
        .find(|s| s.into_lst_state().mint == JUPSOL_MINT.to_bytes())
        .unwrap();
    U8BoolMut(&mut unsafe { lst_state.as_lst_state_mut() }.is_input_disabled).set_true();

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
        amount: 696_969,
        accs,
    };

    let mut bef = prefix_am.0.into_iter().chain(pp_am).chain(inp_am).collect();
    fill_swap_prog_accs(&mut bef, &accs);

    SVM.with(|svm| {
        swap_exact_in_v2_test(
            svm,
            &args,
            &bef,
            Some(Inf1CtlCustomProgErr(Inf1CtlErr::LstInputDisabled)),
        );
    });
}

#[test]
fn swap_exact_in_pool_rebalancing_fixture() {
    let mut prefix_am = jupsol_to_wsol_prefix_fixtures();
    let prefix_keys = IxPreAccs(prefix_am.0.each_ref().map(|(addr, _)| addr.to_bytes()));
    let out_accs = SvcCalcAccsAg::Wsol(WsolCalcAccs);
    let (pp_accs, pp_am) = flatslab_fixture_suf_accs();
    let (inp_accs, inp_am) = jupsol_fixture_svc_suf_accs();

    let mut pool_state = VerPoolState::from_acc_data(&prefix_am.pool_state().1.data);
    *pool_state.is_rebalancing_mut() = 1;
    prefix_am.pool_state_mut().1 = pool_state.into_account();

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
        amount: 696_969,
        accs,
    };

    let mut bef = prefix_am.0.into_iter().chain(pp_am).chain(inp_am).collect();
    fill_swap_prog_accs(&mut bef, &accs);

    SVM.with(|svm| {
        swap_exact_in_v2_test(
            svm,
            &args,
            &bef,
            Some(Inf1CtlCustomProgErr(Inf1CtlErr::PoolRebalancing)),
        );
    });
}

#[test]
fn swap_exact_in_pool_disabled_fixture() {
    let mut prefix_am = jupsol_to_wsol_prefix_fixtures();
    let prefix_keys = IxPreAccs(prefix_am.0.each_ref().map(|(addr, _)| addr.to_bytes()));
    let out_accs = SvcCalcAccsAg::Wsol(WsolCalcAccs);
    let (pp_accs, pp_am) = flatslab_fixture_suf_accs();
    let (inp_accs, inp_am) = jupsol_fixture_svc_suf_accs();

    let mut pool_state = VerPoolState::from_acc_data(&prefix_am.pool_state().1.data);
    *pool_state.is_disabled_mut() = 1;
    prefix_am.pool_state_mut().1 = pool_state.into_account();

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
        amount: 696_969,
        accs,
    };

    let mut bef = prefix_am.0.into_iter().chain(pp_am).chain(inp_am).collect();
    fill_swap_prog_accs(&mut bef, &accs);

    SVM.with(|svm| {
        swap_exact_in_v2_test(
            svm,
            &args,
            &bef,
            Some(Inf1CtlCustomProgErr(Inf1CtlErr::PoolDisabled)),
        );
    });
}

#[test]
fn swap_exact_in_slippage_tolerance_exceeded_fixture() {
    let amount = 10_000;
    let limit = 12_000;
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
        limit,
        amount,
        accs,
    };

    let mut bef = prefix_am.0.into_iter().chain(pp_am).chain(inp_am).collect();
    fill_swap_prog_accs(&mut bef, &accs);

    SVM.with(|svm| {
        swap_exact_in_v2_test(
            svm,
            &args,
            &bef,
            Some(Inf1CtlCustomProgErr(Inf1CtlErr::SlippageToleranceExceeded)),
        );
    });
}

#[test]
fn swap_exact_in_same_lst_fixture() {
    let amount = 10_000;
    let limit = 10_000;
    let prefix_am = IxPreAccs(
        NewSwapExactOutV2IxPreAccsBuilder::start()
            .with_signer("wsol-token-acc-owner")
            .with_pool_state("pool-state")
            .with_lst_state_list("lst-state-list")
            .with_inp_acc("wsol-token-acc")
            .with_inp_mint("wsol-mint")
            .with_inp_pool_reserves("wsol-reserves")
            .with_out_acc("wsol-token-acc")
            .with_out_mint("wsol-mint")
            .with_out_pool_reserves("wsol-reserves")
            // filler
            .with_inp_token_program("wsol-mint")
            .with_out_token_program("wsol-mint")
            .build()
            .0
            .map(|n| KeyedUiAccount::from_test_fixtures_json(n).into_keyed_account()),
    )
    .with_inp_token_program(mollusk_svm_programs_token::token::keyed_account())
    .with_out_token_program(mollusk_svm_programs_token::token::keyed_account());
    let prefix_keys = IxPreAccs(prefix_am.0.each_ref().map(|(addr, _)| addr.to_bytes()));
    let [inp_calc, out_calc] = core::array::from_fn(|_| SvcCalcAccsAg::Wsol(WsolCalcAccs));
    let (pp_accs, pp_am) = flatslab_fixture_suf_accs();

    let accs = Accs {
        ix_prefix: prefix_keys,
        inp_calc_prog: *SvcAgTy::Wsol(()).svc_program_id(),
        inp_calc,
        out_calc_prog: *SvcAgTy::Wsol(()).svc_program_id(),
        out_calc,
        pricing_prog: *PricingAgTy::FlatSlab(()).program_id(),
        pricing: PricingAg::FlatSlab(pp_accs),
    };
    let args = Args {
        inp_lst_index: WSOL_FIXTURE_LST_IDX.try_into().unwrap(),
        out_lst_index: WSOL_FIXTURE_LST_IDX.try_into().unwrap(),
        limit,
        amount,
        accs,
    };

    let mut bef = prefix_am.0.into_iter().chain(pp_am).collect();
    fill_swap_prog_accs(&mut bef, &accs);

    SVM.with(|svm| {
        swap_exact_in_v2_test(
            svm,
            &args,
            &bef,
            Some(Inf1CtlCustomProgErr(Inf1CtlErr::SwapSameLst)),
        );
    });
}
