use inf1_ctl_jiminy::{
    accounts::{lst_state_list::LstStatePackedListMut, pool_state::PoolStatePacked},
    err::Inf1CtlErr,
    instructions::swap::IxArgs,
    keys::{LST_STATE_LIST_ID, POOL_STATE_ID},
    program_err::Inf1CtlCustomProgErr,
};
use inf1_pp_ag_core::{
    inf1_pp_flatslab_core::{
        instructions::pricing::{FlatSlabPpAccs, NewIxSufAccsBuilder},
        keys::SLAB_ID,
    },
    instructions::PriceExactInAccsAg,
    PricingAgTy,
};
use inf1_svc_ag_core::{inf1_svc_lido_core::solido_legacy_core::TOKENKEG_PROGRAM, SvcAgTy};
use inf1_svc_jiminy::traits::SolValCalcAccs;
use inf1_test_utils::{
    assert_jiminy_prog_err, lst_state_list_account, mollusk_exec, pool_state_account, AccountMap,
    KeyedUiAccount, JUPSOL_FIXTURE_LST_IDX, JUPSOL_MINT,
};
use mollusk_svm::result::{InstructionResult, ProgramResult};

use crate::common::{jupsol_fixtures_svc_suf, SVM};

#[test]
fn swap_exact_in_jupsol_msol_fixture() {
    let (ix_prefix, ix, builder, inp_calc, out_calc, pricing) =
        get_jupsol_msol_setup(10000, 8000, SwapIxType::ExactIn);

    let accounts = swap_ix_fixtures_accounts_opt(&builder);

    let (
        _,
        InstructionResult {
            program_result,
            resulting_accounts: resulting_vec,
            ..
        },
    ) = SVM.with(|svm| mollusk_exec(svm, &ix, &accounts));

    let resulting_accounts: AccountMap = resulting_vec.into_iter().collect();

    assert_eq!(program_result, ProgramResult::Success);

    assert_correct_swap(
        SwapIxType::ExactIn,
        &accounts,
        &resulting_accounts,
        10000,
        ix_prefix,
        inp_calc,
        out_calc,
        pricing,
    );
}

#[test]
fn swap_exact_in_input_disabled_fixture() {
    let (_, ix, builder, ..) = get_jupsol_msol_setup(10000, 8000, SwapIxType::ExactIn);

    let mut accounts = swap_ix_fixtures_accounts_opt(&builder);

    let (_, mut lst_state_list_acc) =
        KeyedUiAccount::from_test_fixtures_json("lst-state-list.json").into_keyed_account();

    let lst_state_list = LstStatePackedListMut::of_acc_data(&mut lst_state_list_acc.data).unwrap();
    lst_state_list.0.iter_mut().for_each(|s| {
        let lst_state = unsafe { s.as_lst_state_mut() };
        lst_state.is_input_disabled = 1;
    });

    accounts.insert(
        LST_STATE_LIST_ID.into(),
        lst_state_list_account(lst_state_list.as_packed_list().as_acc_data().to_vec()),
    );

    let (_, InstructionResult { program_result, .. }) =
        SVM.with(|svm| mollusk_exec(svm, &ix, &accounts));

    assert_jiminy_prog_err::<Inf1CtlCustomProgErr>(
        &program_result,
        Inf1CtlCustomProgErr(Inf1CtlErr::LstInputDisabled),
    );
}

#[test]
fn swap_exact_in_pool_rebalancing() {
    let (_, ix, builder, ..) = get_jupsol_msol_setup(10000, 8000, SwapIxType::ExactIn);

    let mut accounts = swap_ix_fixtures_accounts_opt(&builder);

    let (_, pool_state_acc) =
        KeyedUiAccount::from_test_fixtures_json("pool-state.json").into_keyed_account();

    let mut pool_state_data = pool_state_acc.data.try_into().unwrap();
    let pool_state_mut = PoolStatePacked::of_acc_data_arr_mut(&mut pool_state_data);

    let pool_state = unsafe { pool_state_mut.as_pool_state_mut() };
    pool_state.is_rebalancing = 1;

    accounts.insert(POOL_STATE_ID.into(), pool_state_account(*pool_state));

    let (_, InstructionResult { program_result, .. }) =
        SVM.with(|svm| mollusk_exec(svm, &ix, &accounts));

    assert_jiminy_prog_err::<Inf1CtlCustomProgErr>(
        &program_result,
        Inf1CtlCustomProgErr(Inf1CtlErr::PoolRebalancing),
    );
}

#[test]
fn swap_exact_in_pool_disabled() {
    let (_, ix, builder, ..) = get_jupsol_msol_setup(10000, 8000, SwapIxType::ExactIn);

    let mut accounts = swap_ix_fixtures_accounts_opt(&builder);

    let (_, pool_state_acc) =
        KeyedUiAccount::from_test_fixtures_json("pool-state.json").into_keyed_account();

    let mut pool_state_data = pool_state_acc.data.try_into().unwrap();
    let pool_state_mut = PoolStatePacked::of_acc_data_arr_mut(&mut pool_state_data);

    let pool_state = unsafe { pool_state_mut.as_pool_state_mut() };
    pool_state.is_disabled = 1;

    accounts.insert(POOL_STATE_ID.into(), pool_state_account(*pool_state));

    let (_, InstructionResult { program_result, .. }) =
        SVM.with(|svm| mollusk_exec(svm, &ix, &accounts));

    assert_jiminy_prog_err::<Inf1CtlCustomProgErr>(
        &program_result,
        Inf1CtlCustomProgErr(Inf1CtlErr::PoolDisabled),
    );
}

#[test]
fn swap_exact_in_slippage_tolerance_exceeded() {
    let (_, ix, builder, ..) = get_jupsol_msol_setup(10000, 9000, SwapIxType::ExactIn);

    let accounts = swap_ix_fixtures_accounts_opt(&builder);

    let (_, InstructionResult { program_result, .. }) =
        SVM.with(|svm| mollusk_exec(svm, &ix, &accounts));

    assert_jiminy_prog_err::<Inf1CtlCustomProgErr>(
        &program_result,
        Inf1CtlCustomProgErr(Inf1CtlErr::SlippageToleranceExceeded),
    );
}

#[test]
fn swap_exact_in_same_lst() {
    let (jupsol_token_acc_owner_pk, _) =
        KeyedUiAccount::from_test_fixtures_json("jupsol-token-acc-owner.json").into_keyed_account();
    let (jupsol_lst_acc_pk, _) =
        KeyedUiAccount::from_test_fixtures_json("jupsol-token-acc.json").into_keyed_account();

    let ix_prefix = swap_ix_pre_keys_owned(
        jupsol_token_acc_owner_pk.to_bytes(),
        &TOKENKEG_PROGRAM,
        JUPSOL_MINT.to_bytes(),
        jupsol_lst_acc_pk.to_bytes(),
        &TOKENKEG_PROGRAM,
        JUPSOL_MINT.to_bytes(),
        jupsol_lst_acc_pk.to_bytes(),
    );

    let builder = SwapKeysBuilder {
        ix_prefix,
        inp_calc_prog: *SvcAgTy::SanctumSplMulti(()).svc_program_id(),
        inp_calc: jupsol_fixtures_svc_suf(),
        out_calc_prog: *SvcAgTy::SanctumSplMulti(()).svc_program_id(),
        out_calc: jupsol_fixtures_svc_suf(),
        pricing_prog: *PricingAgTy::FlatSlab(()).program_id(),
        pricing: PriceExactInAccsAg::FlatSlab(FlatSlabPpAccs(
            NewIxSufAccsBuilder::start().with_slab(SLAB_ID).build(),
        )),
    };

    let ix = get_swap_ix(
        &builder,
        IxArgs {
            amount: 10000,
            limit: 8000,
            inp_lst_index: JUPSOL_FIXTURE_LST_IDX as u32,
            out_lst_index: JUPSOL_FIXTURE_LST_IDX as u32,
            inp_lst_value_calc_accs: jupsol_fixtures_svc_suf().suf_len() + 1,
            out_lst_value_calc_accs: msol_fixtures_svc_suf().suf_len() + 1,
        },
        SwapIxType::ExactIn,
    );

    let accounts = swap_ix_fixtures_accounts_opt(&builder);

    let (_, InstructionResult { program_result, .. }) =
        SVM.with(|svm| mollusk_exec(svm, &ix, &accounts));

    assert_jiminy_prog_err::<Inf1CtlCustomProgErr>(
        &program_result,
        Inf1CtlCustomProgErr(Inf1CtlErr::SwapSameLst),
    );
}
