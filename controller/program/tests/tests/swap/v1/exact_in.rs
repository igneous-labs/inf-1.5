use expect_test::expect;
use inf1_ctl_jiminy::instructions::swap::v1::{
    exact_in::{NewSwapExactInIxPreAccsBuilder, SwapExactInIxData},
    IxPreAccs,
};
use inf1_pp_ag_core::{instructions::PriceExactInAccsAg, PricingAgTy};
use inf1_std::{
    instructions::swap::v1::exact_in::{
        swap_exact_in_ix_is_signer, swap_exact_in_ix_is_writer, swap_exact_in_ix_keys_owned,
    },
    quote::Quote,
};
use inf1_svc_ag_core::{SvcAg, SvcAgTy};
use inf1_test_utils::{
    assert_jiminy_prog_err, flatslab_fixture_suf_accs, jupsol_fixture_svc_suf_accs,
    keys_signer_writable_to_metas, mollusk_exec, msol_fixture_svc_suf_accs, AccountMap,
    KeyedUiAccount, JUPSOL_FIXTURE_LST_IDX, MSOL_FIXTURE_LST_IDX,
};
use jiminy_cpi::program_error::ProgramError;
use mollusk_svm::Mollusk;
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

use crate::{
    common::SVM,
    tests::swap::{
        common::{add_swap_prog_accs, assert_correct_swap_exact_in_v2},
        v1::args_to_v2,
        V1Accs, V1Args,
    },
};

type Args = V1Args<PriceExactInAccsAg>;

fn to_ix(args: &Args) -> Instruction {
    let accounts = keys_signer_writable_to_metas(
        swap_exact_in_ix_keys_owned(&args.accs).seq(),
        swap_exact_in_ix_is_signer(&args.accs).seq(),
        swap_exact_in_ix_is_writer(&args.accs).seq(),
    );
    Instruction {
        program_id: Pubkey::new_from_array(inf1_ctl_jiminy::ID),
        accounts,
        data: SwapExactInIxData::new(&args.to_full()).as_buf().into(),
    }
}

/// Returns `None` if expected_err is `Some`
fn swap_exact_in_test(
    svm: &Mollusk,
    args: &Args,
    bef: &AccountMap,
    expected_err: Option<impl Into<ProgramError>>,
) -> Option<Quote> {
    let ix = to_ix(args);

    let result = mollusk_exec(svm, &[ix], bef);

    match expected_err {
        None => {
            let aft = result.unwrap().resulting_accounts;
            let clock = &svm.sysvars.clock;
            Some(assert_correct_swap_exact_in_v2(
                bef,
                &aft,
                &args_to_v2(*args),
                clock.epoch,
                clock.slot,
            ))
        }
        Some(e) => {
            assert_jiminy_prog_err(&result.unwrap_err(), e);
            None
        }
    }
}

#[test]
fn swap_exact_in_jupsol_to_msol_fixture() {
    const AMOUNT: u64 = 8_000;

    let prefix_am = NewSwapExactInIxPreAccsBuilder::start()
        .with_signer("jupsol-token-acc-owner")
        .with_pool_state("pool-state")
        .with_lst_state_list("lst-state-list")
        .with_inp_lst_acc("jupsol-token-acc")
        .with_inp_lst_mint("jupsol-mint")
        .with_inp_pool_reserves("jupsol-reserves")
        .with_out_lst_acc("msol-token-acc")
        .with_out_lst_mint("msol-mint")
        .with_out_pool_reserves("msol-reserves")
        .with_inp_lst_token_program("tokenkeg")
        .with_out_lst_token_program("tokenkeg")
        .with_protocol_fee_accumulator("msol-pf-accum")
        .build()
        .0
        .map(|n| KeyedUiAccount::from_test_fixtures_json(n).into_keyed_account());
    let prefix_keys = IxPreAccs(prefix_am.each_ref().map(|(addr, _)| addr.to_bytes()));
    let (pp_accs, pp_am) = flatslab_fixture_suf_accs();
    let (inp_accs, inp_am) = jupsol_fixture_svc_suf_accs();
    let (out_accs, out_am) = msol_fixture_svc_suf_accs();

    let accs = V1Accs {
        ix_prefix: prefix_keys,
        inp_calc_prog: *SvcAgTy::SanctumSplMulti(()).svc_program_id(),
        inp_calc: SvcAg::SanctumSplMulti(inp_accs),
        out_calc_prog: *SvcAgTy::Marinade(()).svc_program_id(),
        out_calc: SvcAg::Marinade(out_accs),
        pricing_prog: *PricingAgTy::FlatSlab(()).program_id(),
        pricing: PriceExactInAccsAg::FlatSlab(pp_accs),
    };
    let args = V1Args {
        inp_lst_index: JUPSOL_FIXTURE_LST_IDX.try_into().unwrap(),
        out_lst_index: MSOL_FIXTURE_LST_IDX.try_into().unwrap(),
        limit: 0,
        amount: AMOUNT,
        accs,
    };

    let mut bef = prefix_am
        .into_iter()
        .chain(pp_am)
        .chain(inp_am)
        .chain(out_am)
        .collect();
    add_swap_prog_accs(&mut bef, &accs);

    let Quote { inp, out, fee, .. } =
        SVM.with(|svm| swap_exact_in_test(svm, &args, &bef, None::<ProgramError>).unwrap());

    expect![[r#"
        (
            8000,
            6842,
            27,
        )
    "#]]
    .assert_debug_eq(&(inp, out, fee));
}

// #[test]
// fn swap_exact_in_input_disabled_fixture() {
//     let (_, ix, builder, ..) = get_jupsol_msol_setup(10000, 8000, SwapIxType::ExactIn);

//     let mut accounts = swap_ix_fixtures_accounts_opt(&builder);

//     let (_, mut lst_state_list_acc) =
//         KeyedUiAccount::from_test_fixtures_json("lst-state-list.json").into_keyed_account();

//     let lst_state_list = LstStatePackedListMut::of_acc_data(&mut lst_state_list_acc.data).unwrap();
//     lst_state_list.0.iter_mut().for_each(|s| {
//         let lst_state = unsafe { s.as_lst_state_mut() };
//         lst_state.is_input_disabled = 1;
//     });

//     accounts.insert(
//         LST_STATE_LIST_ID.into(),
//         lst_state_list_account(lst_state_list.as_packed_list().as_acc_data().to_vec()),
//     );

//     let (_, InstructionResult { program_result, .. }) =
//         SVM.with(|svm| mollusk_exec(svm, &ix, &accounts));

//     assert_jiminy_prog_err::<Inf1CtlCustomProgErr>(
//         &program_result,
//         Inf1CtlCustomProgErr(Inf1CtlErr::LstInputDisabled),
//     );
// }

// #[test]
// fn swap_exact_in_pool_rebalancing() {
//     let (_, ix, builder, ..) = get_jupsol_msol_setup(10000, 8000, SwapIxType::ExactIn);

//     let mut accounts = swap_ix_fixtures_accounts_opt(&builder);

//     let (_, pool_state_acc) =
//         KeyedUiAccount::from_test_fixtures_json("pool-state.json").into_keyed_account();

//     let mut pool_state_data = pool_state_acc.data.try_into().unwrap();
//     let pool_state_mut = PoolStatePacked::of_acc_data_arr_mut(&mut pool_state_data);

//     let pool_state = unsafe { pool_state_mut.as_pool_state_mut() };
//     pool_state.is_rebalancing = 1;

//     accounts.insert(POOL_STATE_ID.into(), pool_state_account(*pool_state));

//     let (_, InstructionResult { program_result, .. }) =
//         SVM.with(|svm| mollusk_exec(svm, &ix, &accounts));

//     assert_jiminy_prog_err::<Inf1CtlCustomProgErr>(
//         &program_result,
//         Inf1CtlCustomProgErr(Inf1CtlErr::PoolRebalancing),
//     );
// }

// #[test]
// fn swap_exact_in_pool_disabled() {
//     let (_, ix, builder, ..) = get_jupsol_msol_setup(10000, 8000, SwapIxType::ExactIn);

//     let mut accounts = swap_ix_fixtures_accounts_opt(&builder);

//     let (_, pool_state_acc) =
//         KeyedUiAccount::from_test_fixtures_json("pool-state.json").into_keyed_account();

//     let mut pool_state_data = pool_state_acc.data.try_into().unwrap();
//     let pool_state_mut = PoolStatePacked::of_acc_data_arr_mut(&mut pool_state_data);

//     let pool_state = unsafe { pool_state_mut.as_pool_state_mut() };
//     pool_state.is_disabled = 1;

//     accounts.insert(POOL_STATE_ID.into(), pool_state_account(*pool_state));

//     let (_, InstructionResult { program_result, .. }) =
//         SVM.with(|svm| mollusk_exec(svm, &ix, &accounts));

//     assert_jiminy_prog_err::<Inf1CtlCustomProgErr>(
//         &program_result,
//         Inf1CtlCustomProgErr(Inf1CtlErr::PoolDisabled),
//     );
// }

// #[test]
// fn swap_exact_in_slippage_tolerance_exceeded() {
//     let (_, ix, builder, ..) = get_jupsol_msol_setup(10000, 9000, SwapIxType::ExactIn);

//     let accounts = swap_ix_fixtures_accounts_opt(&builder);

//     let (_, InstructionResult { program_result, .. }) =
//         SVM.with(|svm| mollusk_exec(svm, &ix, &accounts));

//     assert_jiminy_prog_err::<Inf1CtlCustomProgErr>(
//         &program_result,
//         Inf1CtlCustomProgErr(Inf1CtlErr::SlippageToleranceExceeded),
//     );
// }

// #[test]
// fn swap_exact_in_same_lst() {
//     let (jupsol_token_acc_owner_pk, _) =
//         KeyedUiAccount::from_test_fixtures_json("jupsol-token-acc-owner.json").into_keyed_account();
//     let (jupsol_lst_acc_pk, _) =
//         KeyedUiAccount::from_test_fixtures_json("jupsol-token-acc.json").into_keyed_account();

//     let ix_prefix = swap_ix_pre_keys_owned(
//         jupsol_token_acc_owner_pk.to_bytes(),
//         &TOKENKEG_PROGRAM,
//         JUPSOL_MINT.to_bytes(),
//         jupsol_lst_acc_pk.to_bytes(),
//         &TOKENKEG_PROGRAM,
//         JUPSOL_MINT.to_bytes(),
//         jupsol_lst_acc_pk.to_bytes(),
//     );

//     let builder = SwapKeysBuilder {
//         ix_prefix,
//         inp_calc_prog: *SvcAgTy::SanctumSplMulti(()).svc_program_id(),
//         inp_calc: jupsol_fixtures_svc_suf(),
//         out_calc_prog: *SvcAgTy::SanctumSplMulti(()).svc_program_id(),
//         out_calc: jupsol_fixtures_svc_suf(),
//         pricing_prog: *PricingAgTy::FlatSlab(()).program_id(),
//         pricing: PriceExactInAccsAg::FlatSlab(FlatSlabPpAccs(
//             NewIxSufAccsBuilder::start().with_slab(SLAB_ID).build(),
//         )),
//     };

//     let ix = get_swap_ix(
//         &builder,
//         IxArgs {
//             amount: 10000,
//             limit: 8000,
//             inp_lst_index: JUPSOL_FIXTURE_LST_IDX as u32,
//             out_lst_index: JUPSOL_FIXTURE_LST_IDX as u32,
//             inp_lst_value_calc_accs: jupsol_fixtures_svc_suf().suf_len() + 1,
//             out_lst_value_calc_accs: msol_fixtures_svc_suf().suf_len() + 1,
//         },
//         SwapIxType::ExactIn,
//     );

//     let accounts = swap_ix_fixtures_accounts_opt(&builder);

//     let (_, InstructionResult { program_result, .. }) =
//         SVM.with(|svm| mollusk_exec(svm, &ix, &accounts));

//     assert_jiminy_prog_err::<Inf1CtlCustomProgErr>(
//         &program_result,
//         Inf1CtlCustomProgErr(Inf1CtlErr::SwapSameLst),
//     );
// }
