use inf1_core::{
    instructions::swap::exact_in::{
        swap_exact_in_ix_is_signer, swap_exact_in_ix_is_writer, swap_exact_in_ix_keys_owned,
        SwapExactInIxAccs,
    },
    quote::swap::{exact_in::quote_exact_in, SwapQuoteArgs},
};
use inf1_ctl_jiminy::{
    accounts::{
        lst_state_list::{LstStatePackedList, LstStatePackedListMut},
        pool_state::PoolStatePacked,
    },
    err::Inf1CtlErr,
    instructions::swap::exact_in::{
        NewSwapExactInIxPreAccsBuilder, SwapExactInIxArgs, SwapExactInIxData,
        SwapExactInIxPreKeysOwned,
    },
    keys::{LST_STATE_LIST_ID, POOL_STATE_ID},
    program_err::Inf1CtlCustomProgErr,
    ID,
};
use inf1_pp_ag_core::{
    inf1_pp_flatslab_core::{
        instructions::pricing::{FlatSlabPpAccs, NewIxSufAccsBuilder},
        keys::SLAB_ID,
    },
    instructions::PriceExactInAccsAg,
    PricingAgTy,
};
use inf1_pp_core::{pair::Pair, traits::main::PriceExactIn};
use inf1_pp_flatslab_std::FlatSlabPricing;
use inf1_svc_ag_core::{
    inf1_svc_lido_core::solido_legacy_core::TOKENKEG_PROGRAM,
    inf1_svc_marinade_core::{
        calc::MarinadeCalc,
        sanctum_marinade_liquid_staking_core::{State, MSOL_MINT_ADDR},
    },
    inf1_svc_spl_core::{calc::SplCalc, sanctum_spl_stake_pool_core::StakePool},
    instructions::SvcCalcAccsAg,
    SvcAgTy,
};
use inf1_svc_jiminy::traits::{SolValCalc, SolValCalcAccs};
use inf1_test_utils::{
    acc_bef_aft, assert_jiminy_prog_err, find_pool_reserves_ata, find_protocol_fee_accumulator_ata,
    fixtures_accounts_opt_cloned, keys_signer_writable_to_metas, lst_state_list_account,
    pool_state_account, upsert_account, KeyedUiAccount, PkAccountTup, JUPSOL_FIXTURE_LST_IDX,
    JUPSOL_MINT, MSOL_FIXTURE_LST_IDX,
};
use mollusk_svm::result::{InstructionResult, ProgramResult};
use sanctum_spl_token_jiminy::sanctum_spl_token_core::state::account::RawTokenAccount;
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

use crate::common::{jupsol_fixtures_svc_suf, msol_fixtures_svc_suf, SVM};

type SwapExactInKeysBuilder = SwapExactInIxAccs<
    [u8; 32],
    SwapExactInIxPreKeysOwned,
    SvcCalcAccsAg,
    SvcCalcAccsAg,
    PriceExactInAccsAg,
>;

fn swap_exact_in_ix_pre_keys_owned(
    signer: [u8; 32],
    inp_token_program: &[u8; 32],
    inp_mint: [u8; 32],
    inp_lst_acc: [u8; 32],
    out_token_program: &[u8; 32],
    out_mint: [u8; 32],
    out_lst_acc: [u8; 32],
) -> SwapExactInIxPreKeysOwned {
    NewSwapExactInIxPreAccsBuilder::start()
        .with_lst_state_list(LST_STATE_LIST_ID)
        .with_pool_state(POOL_STATE_ID)
        .with_inp_lst_mint(inp_mint)
        .with_inp_lst_acc(inp_lst_acc)
        .with_inp_lst_token_program(*inp_token_program)
        .with_out_lst_mint(out_mint)
        .with_out_lst_acc(out_lst_acc)
        .with_out_lst_token_program(*out_token_program)
        .with_protocol_fee_accumulator(
            find_protocol_fee_accumulator_ata(out_token_program, &out_mint)
                .0
                .to_bytes(),
        )
        .with_inp_pool_reserves(
            find_pool_reserves_ata(inp_token_program, &inp_mint)
                .0
                .to_bytes(),
        )
        .with_out_pool_reserves(
            find_pool_reserves_ata(out_token_program, &out_mint)
                .0
                .to_bytes(),
        )
        .with_signer(signer)
        .build()
}

fn swap_exact_in_ix(builder: &SwapExactInKeysBuilder, args: SwapExactInIxArgs) -> Instruction {
    let accounts = keys_signer_writable_to_metas(
        swap_exact_in_ix_keys_owned(builder).seq(),
        swap_exact_in_ix_is_signer(builder).seq(),
        swap_exact_in_ix_is_writer(builder).seq(),
    );

    Instruction {
        program_id: Pubkey::new_from_array(ID),
        accounts,
        data: SwapExactInIxData::new(args).as_buf().into(),
    }
}

fn get_jupsol_msol_setup(
    amount: u64,
    limit: u64,
) -> (
    SwapExactInIxPreKeysOwned,
    Instruction,
    SwapExactInKeysBuilder,
    impl SolValCalc,
    impl SolValCalc,
    impl PriceExactIn,
) {
    let (jupsol_token_acc_owner_pk, _) =
        KeyedUiAccount::from_test_fixtures_json("jupsol-token-acc-owner.json").into_keyed_account();
    let (jupsol_lst_acc_pk, _) =
        KeyedUiAccount::from_test_fixtures_json("jupsol-token-acc.json").into_keyed_account();

    let (msol_lst_acc_pk, _) =
        KeyedUiAccount::from_test_fixtures_json("msol-token-acc.json").into_keyed_account();

    let (_, jupsol_pool_acc) =
        KeyedUiAccount::from_test_fixtures_json("jupsol-pool.json").into_keyed_account();
    let (_, marinade_pool_acc) =
        KeyedUiAccount::from_test_fixtures_json("msol-pool.json").into_keyed_account();
    let (_, slab_acc) =
        KeyedUiAccount::from_test_fixtures_json("flatslab-slab.json").into_keyed_account();

    let jupsol_stakepool = StakePool::borsh_de(jupsol_pool_acc.data.as_slice()).unwrap();
    let marinade_stakepool = State::borsh_de(marinade_pool_acc.data.as_slice()).unwrap();

    let inp_calc = SplCalc::new(&jupsol_stakepool, 0);
    let out_calc = MarinadeCalc::new(&marinade_stakepool);
    let pricing = FlatSlabPricing::new(slab_acc.data.into_boxed_slice())
        .flat_slab_swap_pricing_for(&Pair {
            inp: &JUPSOL_MINT.to_bytes(),
            out: &MSOL_MINT_ADDR,
        })
        .unwrap();

    let ix_prefix = swap_exact_in_ix_pre_keys_owned(
        jupsol_token_acc_owner_pk.to_bytes(),
        &TOKENKEG_PROGRAM,
        JUPSOL_MINT.to_bytes(),
        jupsol_lst_acc_pk.to_bytes(),
        &TOKENKEG_PROGRAM,
        MSOL_MINT_ADDR,
        msol_lst_acc_pk.to_bytes(),
    );

    let builder = SwapExactInKeysBuilder {
        ix_prefix,
        inp_calc_prog: *SvcAgTy::SanctumSplMulti(()).svc_program_id(),
        inp_calc: jupsol_fixtures_svc_suf(),
        out_calc_prog: *SvcAgTy::Marinade(()).svc_program_id(),
        out_calc: msol_fixtures_svc_suf(),
        pricing_prog: *PricingAgTy::FlatSlab(()).program_id(),
        pricing: PriceExactInAccsAg::FlatSlab(FlatSlabPpAccs(
            NewIxSufAccsBuilder::start().with_slab(SLAB_ID).build(),
        )),
    };

    let ix = swap_exact_in_ix(
        &builder,
        SwapExactInIxArgs {
            amount,
            limit,
            inp_lst_index: JUPSOL_FIXTURE_LST_IDX as u32,
            out_lst_index: MSOL_FIXTURE_LST_IDX as u32,
            inp_lst_value_calc_accs: jupsol_fixtures_svc_suf().suf_len() + 1,
            out_lst_value_calc_accs: msol_fixtures_svc_suf().suf_len() + 1,
        },
    );

    (ix_prefix, ix, builder, inp_calc, out_calc, pricing)
}

fn swap_exact_in_fixtures_accounts_opt(builder: &SwapExactInKeysBuilder) -> Vec<PkAccountTup> {
    fixtures_accounts_opt_cloned(swap_exact_in_ix_keys_owned(builder).seq().copied()).collect()
}

fn assert_correct_swap_exact_in<T: SolValCalc, O: SolValCalc, P: PriceExactIn>(
    bef: &[PkAccountTup],
    aft: &[PkAccountTup],
    amount: u64,
    inp_mint: &[u8; 32],
    out_mint: &[u8; 32],
    inp_lst_acc: [u8; 32],
    out_lst_acc: [u8; 32],
    pf_accum_acc: [u8; 32],
    out_pool_reserves_acc: [u8; 32],
    inp_calc: T,
    out_calc: O,
    pricing: P,
) -> i128 {
    let [pools, lst_state_lists, inp_lst_accs, out_lst_accs, protocol_fee_accumulator_accs, out_pool_reserves_accs] =
        [
            POOL_STATE_ID,
            LST_STATE_LIST_ID,
            inp_lst_acc,
            out_lst_acc,
            pf_accum_acc,
            out_pool_reserves_acc,
        ]
        .map(|a| acc_bef_aft(&Pubkey::new_from_array(a), bef, aft));

    let [pool_bef, pool_aft] = pools.each_ref().map(|a| {
        PoolStatePacked::of_acc_data(&a.data)
            .unwrap()
            .into_pool_state()
    });

    let [lst_state_list_bef, lst_state_list_aft] = lst_state_lists
        .each_ref()
        .map(|a| LstStatePackedList::of_acc_data(&a.data).unwrap());

    let [[inp_lst_acc_bef, inp_lst_acc_aft], [out_lst_acc_bef, out_lst_acc_aft], [protocol_fee_accumulator_bef, protocol_fee_accumulator_aft], [out_pool_reserves_bef, _]] =
        [
            inp_lst_accs,
            out_lst_accs,
            protocol_fee_accumulator_accs,
            out_pool_reserves_accs,
        ]
        .map(|accs| {
            accs.each_ref()
                .map(|a| RawTokenAccount::of_acc_data(&a.data).unwrap())
        });

    let quote = quote_exact_in(SwapQuoteArgs {
        amt: amount,
        out_reserves: u64::from_le_bytes(out_pool_reserves_bef.amount),
        trading_protocol_fee_bps: pool_bef.trading_protocol_fee_bps,
        inp_mint: *inp_mint,
        out_mint: *out_mint,
        inp_calc,
        out_calc,
        pricing,
    })
    .unwrap();

    let inp_lst_state_idx = lst_state_list_bef
        .0
        .iter()
        .position(|s| s.into_lst_state().mint == JUPSOL_MINT.to_bytes())
        .unwrap();

    let out_lst_state_idx = lst_state_list_aft
        .0
        .iter()
        .position(|s| s.into_lst_state().mint == MSOL_MINT_ADDR)
        .unwrap();

    let [inp_lst_state_bef, inp_lst_state_aft] =
        [lst_state_list_bef, lst_state_list_aft].map(|l| l.0[inp_lst_state_idx].into_lst_state());
    let [out_lst_state_bef, out_lst_state_aft] =
        [lst_state_list_bef, lst_state_list_aft].map(|l| l.0[out_lst_state_idx].into_lst_state());

    let inp_sol_val_delta =
        i128::from(inp_lst_state_aft.sol_value) - i128::from(inp_lst_state_bef.sol_value);
    let out_sol_val_delta =
        i128::from(out_lst_state_aft.sol_value) - i128::from(out_lst_state_bef.sol_value);

    let total_delta = inp_sol_val_delta + out_sol_val_delta;

    let pool_sol_val_delta =
        i128::from(pool_aft.total_sol_value) - i128::from(pool_bef.total_sol_value);

    assert_eq!(total_delta, pool_sol_val_delta);
    assert!(pool_sol_val_delta > 0);

    let inp_lst_balance_delta =
        u64::from_le_bytes(inp_lst_acc_bef.amount) - u64::from_le_bytes(inp_lst_acc_aft.amount);
    let out_lst_balance_delta =
        u64::from_le_bytes(out_lst_acc_aft.amount) - u64::from_le_bytes(out_lst_acc_bef.amount);
    let protocol_fee_accumulator_balance_delta =
        u64::from_le_bytes(protocol_fee_accumulator_aft.amount)
            - u64::from_le_bytes(protocol_fee_accumulator_bef.amount);

    assert_eq!(inp_lst_balance_delta, quote.0.inp);
    assert_eq!(out_lst_balance_delta, quote.0.out);
    assert_eq!(protocol_fee_accumulator_balance_delta, quote.0.protocol_fee);

    pool_sol_val_delta
}

#[test]
fn swap_exact_in_jupsol_msol_fixture() {
    let (ix_prefix, ix, builder, inp_calc, out_calc, pricing) = get_jupsol_msol_setup(10000, 8000);

    let accounts = swap_exact_in_fixtures_accounts_opt(&builder);

    let InstructionResult {
        program_result,
        resulting_accounts,
        ..
    } = SVM.with(|svm| svm.process_instruction(&ix, &accounts));

    assert_eq!(program_result, ProgramResult::Success);

    assert_correct_swap_exact_in(
        &accounts,
        &resulting_accounts,
        10000,
        &JUPSOL_MINT.to_bytes(),
        &MSOL_MINT_ADDR,
        *ix_prefix.inp_lst_acc(),
        *ix_prefix.out_lst_acc(),
        *ix_prefix.protocol_fee_accumulator(),
        *ix_prefix.out_pool_reserves(),
        inp_calc,
        out_calc,
        pricing,
    );
}

#[test]
fn swap_exact_in_input_disabled_fixture() {
    let (_, ix, builder, ..) = get_jupsol_msol_setup(10000, 8000);

    let mut accounts = swap_exact_in_fixtures_accounts_opt(&builder);

    let (_, mut lst_state_list_acc) =
        KeyedUiAccount::from_test_fixtures_json("lst-state-list.json").into_keyed_account();

    let lst_state_list = LstStatePackedListMut::of_acc_data(&mut lst_state_list_acc.data).unwrap();
    lst_state_list.0.iter_mut().for_each(|s| {
        let lst_state = unsafe { s.as_lst_state_mut() };
        lst_state.is_input_disabled = 1;
    });

    upsert_account(
        &mut accounts,
        (
            LST_STATE_LIST_ID.into(),
            lst_state_list_account(lst_state_list.as_packed_list().as_acc_data().to_vec()),
        ),
    );

    let InstructionResult { program_result, .. } =
        SVM.with(|svm| svm.process_instruction(&ix, &accounts));

    assert_jiminy_prog_err::<Inf1CtlCustomProgErr>(
        &program_result,
        Inf1CtlCustomProgErr(Inf1CtlErr::LstInputDisabled),
    );
}

#[test]
fn swap_exact_in_pool_rebalancing() {
    let (_, ix, builder, ..) = get_jupsol_msol_setup(10000, 8000);

    let mut accounts = swap_exact_in_fixtures_accounts_opt(&builder);

    let (_, pool_state_acc) =
        KeyedUiAccount::from_test_fixtures_json("pool-state.json").into_keyed_account();

    let mut pool_state_data = pool_state_acc.data.try_into().unwrap();
    let pool_state_mut = PoolStatePacked::of_acc_data_arr_mut(&mut pool_state_data);

    let pool_state = unsafe { pool_state_mut.as_pool_state_mut() };
    pool_state.is_rebalancing = 1;

    upsert_account(
        &mut accounts,
        (POOL_STATE_ID.into(), pool_state_account(*pool_state)),
    );

    let InstructionResult { program_result, .. } =
        SVM.with(|svm| svm.process_instruction(&ix, &accounts));

    assert_jiminy_prog_err::<Inf1CtlCustomProgErr>(
        &program_result,
        Inf1CtlCustomProgErr(Inf1CtlErr::PoolRebalancing),
    );
}

#[test]
fn swap_exact_in_pool_disabled() {
    let (_, ix, builder, ..) = get_jupsol_msol_setup(10000, 8000);

    let mut accounts = swap_exact_in_fixtures_accounts_opt(&builder);

    let (_, pool_state_acc) =
        KeyedUiAccount::from_test_fixtures_json("pool-state.json").into_keyed_account();

    let mut pool_state_data = pool_state_acc.data.try_into().unwrap();
    let pool_state_mut = PoolStatePacked::of_acc_data_arr_mut(&mut pool_state_data);

    let pool_state = unsafe { pool_state_mut.as_pool_state_mut() };
    pool_state.is_disabled = 1;

    upsert_account(
        &mut accounts,
        (POOL_STATE_ID.into(), pool_state_account(*pool_state)),
    );

    let InstructionResult { program_result, .. } =
        SVM.with(|svm| svm.process_instruction(&ix, &accounts));

    assert_jiminy_prog_err::<Inf1CtlCustomProgErr>(
        &program_result,
        Inf1CtlCustomProgErr(Inf1CtlErr::PoolDisabled),
    );
}

#[test]
fn swap_exact_in_slippage_tolerance_exceeded() {
    let (_, ix, builder, ..) = get_jupsol_msol_setup(10000, 9000);

    let accounts = swap_exact_in_fixtures_accounts_opt(&builder);

    let InstructionResult { program_result, .. } =
        SVM.with(|svm| svm.process_instruction(&ix, &accounts));

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

    let ix_prefix = swap_exact_in_ix_pre_keys_owned(
        jupsol_token_acc_owner_pk.to_bytes(),
        &TOKENKEG_PROGRAM,
        JUPSOL_MINT.to_bytes(),
        jupsol_lst_acc_pk.to_bytes(),
        &TOKENKEG_PROGRAM,
        JUPSOL_MINT.to_bytes(),
        jupsol_lst_acc_pk.to_bytes(),
    );

    let builder = SwapExactInKeysBuilder {
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

    let ix = swap_exact_in_ix(
        &builder,
        SwapExactInIxArgs {
            amount: 10000,
            limit: 8000,
            inp_lst_index: JUPSOL_FIXTURE_LST_IDX as u32,
            out_lst_index: JUPSOL_FIXTURE_LST_IDX as u32,
            inp_lst_value_calc_accs: jupsol_fixtures_svc_suf().suf_len() + 1,
            out_lst_value_calc_accs: msol_fixtures_svc_suf().suf_len() + 1,
        },
    );

    let accounts = swap_exact_in_fixtures_accounts_opt(&builder);

    let InstructionResult { program_result, .. } =
        SVM.with(|svm| svm.process_instruction(&ix, &accounts));

    assert_jiminy_prog_err::<Inf1CtlCustomProgErr>(
        &program_result,
        Inf1CtlCustomProgErr(Inf1CtlErr::SwapSameLst),
    );
}
// const fn max_sol_val_no_overflow(pool_total_sol_val: u64, old_lst_state_sol_val: u64) -> u64 {
//     u64::MAX - (pool_total_sol_val - old_lst_state_sol_val)
// }

// fn swap_exact_in_spl_multi_proptest(
//     pool: PoolState,
//     mut lsl: LstStateListData,
//     inp_lsd: LstStateData,
//     inp_stake_pool_addr: [u8; 32],
//     inp_stake_pool: StakePool,
//     out_lsd: LstStateData,
//     out_stake_pool_addr: [u8; 32],
//     out_stake_pool: StakePool,
//     exact_in_amount: u64,
// ) -> TestCaseResult {
//     let inp_lst_idx = lsl.upsert(inp_lsd);
//     let out_lst_idx = lsl.upsert(out_lsd);

//     let LstStateListData {
//         lst_state_list,
//         all_pool_reserves,
//         ..
//     } = lsl;

//     let inp_lst_owner = Pubkey::new_unique();
//     let inp_lst_token_acc_pk = Pubkey::new_unique();

//     let out_lst_owner = Pubkey::new_unique();
//     let out_lst_token_acc_pk = Pubkey::new_unique();

//     let inp_lst_token_acc = mock_token_acc(raw_token_acc(
//         inp_lsd.lst_state.mint,
//         inp_lst_owner.to_bytes(),
//         1_000_000_000,
//     ));

//     let out_lst_token_acc = mock_token_acc(raw_token_acc(
//         out_lsd.lst_state.mint,
//         out_lst_owner.to_bytes(),
//         1_000_000_000,
//     ));

//     let ix_prefix = swap_exact_in_ix_pre_keys_owned(
//         inp_lst_owner.to_bytes(),
//         &TOKENKEG_PROGRAM,
//         inp_lsd.lst_state.mint,
//         inp_lst_token_acc_pk.to_bytes(),
//         &TOKENKEG_PROGRAM,
//         out_lsd.lst_state.mint,
//         out_lst_token_acc_pk.to_bytes(),
//     );

//     let builder = SwapExactInKeysBuilder {
//         ix_prefix,
//         inp_calc_prog: inp_lsd.lst_state.sol_value_calculator,
//         inp_calc: SvcCalcAccsAg::SanctumSplMulti(SanctumSplMultiCalcAccs {
//             stake_pool_addr: inp_stake_pool_addr,
//         }),
//         out_calc_prog: out_lsd.lst_state.sol_value_calculator,
//         out_calc: SvcCalcAccsAg::SanctumSplMulti(SanctumSplMultiCalcAccs {
//             stake_pool_addr: out_stake_pool_addr,
//         }),
//         // TODO: Do we wanna do prop testing over the slab?
//         pricing_prog: pool.pricing_program,
//         pricing: PriceExactInAccsAg::FlatSlab(FlatSlabPpAccs(
//             NewIxSufAccsBuilder::start().with_slab(SLAB_ID).build(),
//         )),
//     };

//     let ix = swap_exact_in_ix(
//         &builder,
//         SwapExactInIxArgs {
//             amount: exact_in_amount,
//             limit: 0, // TODO: How to know what the limit should be?
//             inp_lst_index: inp_lst_idx as u32,
//             out_lst_index: out_lst_idx as u32,
//             inp_lst_value_calc_accs: builder.inp_calc.suf_len() + 1,
//             out_lst_value_calc_accs: builder.out_calc.suf_len() + 1,
//         },
//     );
//     let mut accounts = swap_exact_in_fixtures_accounts_opt(&builder);

//     upsert_account(
//         &mut accounts,
//         (
//             LST_STATE_LIST_ID.into(),
//             lst_state_list_account(lst_state_list),
//         ),
//     );
//     upsert_account(
//         &mut accounts,
//         (POOL_STATE_ID.into(), pool_state_account(pool)),
//     );

//     // Upsert stake pools
//     upsert_account(
//         &mut accounts,
//         (
//             Pubkey::new_from_array(inp_stake_pool_addr),
//             mock_spl_stake_pool(&inp_stake_pool, sanctum_spl_multi::POOL_PROG_ID.into()),
//         ),
//     );

//     upsert_account(
//         &mut accounts,
//         (
//             Pubkey::new_from_array(out_stake_pool_addr),
//             mock_spl_stake_pool(&out_stake_pool, sanctum_spl_multi::POOL_PROG_ID.into()),
//         ),
//     );

//     // Upsert mints
//     upsert_account(
//         &mut accounts,
//         (
//             inp_lsd.lst_state.mint.into(),
//             mock_mint(raw_mint(None, None, u64::MAX, 9)),
//         ),
//     );
//     upsert_account(
//         &mut accounts,
//         (
//             out_lsd.lst_state.mint.into(),
//             mock_mint(raw_mint(None, None, u64::MAX, 9)),
//         ),
//     );

//     // Upsert lst token accs
//     upsert_account(
//         &mut accounts,
//         (inp_lst_token_acc_pk.into(), inp_lst_token_acc),
//     );

//     upsert_account(
//         &mut accounts,
//         (out_lst_token_acc_pk.into(), out_lst_token_acc),
//     );

//     let out_reserves: u64 =
//         max_sol_val_no_overflow(pool.total_sol_value, out_lsd.lst_state.sol_value)
//             / MAX_LAMPORTS_OVER_SUPPLY;

//     // Upsert pool reserves
//     upsert_account(
//         &mut accounts,
//         (
//             Pubkey::new_from_array(*all_pool_reserves.get(&inp_lsd.lst_state.mint).unwrap()),
//             mock_token_acc(raw_token_acc(
//                 inp_lsd.lst_state.mint,
//                 POOL_STATE_ID,
//                 max_sol_val_no_overflow(pool.total_sol_value, inp_lsd.lst_state.sol_value)
//                     / MAX_LAMPORTS_OVER_SUPPLY,
//             )),
//         ),
//     );
//     upsert_account(
//         &mut accounts,
//         (
//             Pubkey::new_from_array(*all_pool_reserves.get(&out_lsd.lst_state.mint).unwrap()),
//             mock_token_acc(raw_token_acc(
//                 out_lsd.lst_state.mint,
//                 POOL_STATE_ID,
//                 out_reserves,
//             )),
//         ),
//     );

//     upsert_account(
//         &mut accounts,
//         (
//             Pubkey::new_from_array(
//                 find_protocol_fee_accumulator(&TOKENKEG_PROGRAM, &out_lsd.lst_state.mint)
//                     .0
//                     .to_bytes(),
//             ),
//             mock_token_acc(raw_token_acc(
//                 out_lsd.lst_state.mint,
//                 POOL_STATE_ID,
//                 100_000_000,
//             )),
//         ),
//     );

//     // Upsert signer
//     upsert_account(&mut accounts, (inp_lst_owner, mock_system_account()));

//     let mut slab_data = [1 as u8; 32 + 2 * size_of::<SlabEntryPacked>()];

//     let sm = SlabMut::of_acc_data(slab_data.as_mut_slice());
//     prop_assert!(sm.is_some());

//     let mut sm = sm.unwrap();
//     let (_, entries) = sm.as_mut();

//     let set_inp_fee_nanos_to = FeeNanos::new(500).unwrap();
//     let set_out_fee_nanos_to = FeeNanos::new(1000).unwrap();

//     let mut lsd_mints = [inp_lsd.lst_state.mint, out_lsd.lst_state.mint];
//     lsd_mints.sort();

//     entries.0.iter_mut().enumerate().for_each(|(i, e)| {
//         *e.mint_mut() = lsd_mints[i];
//         e.set_inp_fee_nanos(set_inp_fee_nanos_to);
//         e.set_out_fee_nanos(set_out_fee_nanos_to);
//     });

//     upsert_account(
//         &mut accounts,
//         (
//             Pubkey::new_from_array(SLAB_ID),
//             mock_slab_account(
//                 slab_data.to_vec(),
//                 Pubkey::new_from_array(pool.pricing_program),
//             ),
//         ),
//     );

//     let InstructionResult {
//         program_result,
//         resulting_accounts,
//         ..
//     } = SVM.with(|svm| svm.process_instruction(&ix, &accounts));

//     // input disabled
//     if inp_lsd.lst_state.is_input_disabled == 1 {
//         assert_jiminy_prog_err::<Inf1CtlCustomProgErr>(
//             &program_result,
//             Inf1CtlCustomProgErr(Inf1CtlErr::LstInputDisabled),
//         );
//     } else {
//         prop_assert_eq!(program_result, ProgramResult::Success);
//     }

//     let [pools, lst_state_lists, inp_lst_accs, out_lst_accs, protocol_fee_accumulator_accs, out_pool_reserves_accs] =
//         [
//             POOL_STATE_ID,
//             LST_STATE_LIST_ID,
//             *ix_prefix.inp_lst_acc(),
//             *ix_prefix.out_lst_acc(),
//             *ix_prefix.protocol_fee_accumulator(),
//             *ix_prefix.out_pool_reserves(),
//         ]
//         .map(|a| acc_bef_aft(&Pubkey::new_from_array(a), &accounts, &resulting_accounts));

//     let [pool_bef, pool_aft] = pools.each_ref().map(|a| {
//         PoolStatePacked::of_acc_data(&a.data)
//             .unwrap()
//             .into_pool_state()
//     });
//     let [lst_state_list_bef, lst_state_list_aft] = lst_state_lists
//         .each_ref()
//         .map(|a| LstStatePackedList::of_acc_data(&a.data).unwrap());

//     let [inp_lst_acc_bef, inp_lst_acc_aft] = inp_lst_accs
//         .each_ref()
//         .map(|a| RawTokenAccount::of_acc_data(&a.data).unwrap());
//     let [out_lst_acc_bef, out_lst_acc_aft] = out_lst_accs
//         .each_ref()
//         .map(|a| RawTokenAccount::of_acc_data(&a.data).unwrap());
//     let [protocol_fee_accumulator_bef, protocol_fee_accumulator_aft] =
//         protocol_fee_accumulator_accs
//             .each_ref()
//             .map(|a| RawTokenAccount::of_acc_data(&a.data).unwrap());

//     let [out_pool_reserves_bef, _] = out_pool_reserves_accs
//         .each_ref()
//         .map(|a| RawTokenAccount::of_acc_data(&a.data).unwrap());

//     let quote = quote_exact_in(SwapQuoteArgs {
//         amt: 10000,
//         out_reserves,
//         trading_protocol_fee_bps: pool.trading_protocol_fee_bps,
//         inp_mint: inp_lsd.lst_state.mint,
//         out_mint: out_lsd.lst_state.mint,
//         inp_calc: SplCalc::new(&inp_stake_pool, 0),
//         out_calc: SplCalc::new(&out_stake_pool, 0),
//         pricing: FlatSlabPricing::new(Box::new(slab_data))
//             .flat_slab_swap_pricing_for(&Pair {
//                 inp: &inp_lsd.lst_state.mint,
//                 out: &out_lsd.lst_state.mint,
//             })
//             .unwrap(),
//     })
//     .unwrap();

//     let inp_lst_state_idx = lst_state_list_bef
//         .0
//         .iter()
//         .position(|s| s.into_lst_state().mint == inp_lsd.lst_state.mint)
//         .unwrap();

//     let out_lst_state_idx = lst_state_list_aft
//         .0
//         .iter()
//         .position(|s| s.into_lst_state().mint == out_lsd.lst_state.mint)
//         .unwrap();

//     let [inp_lst_state_bef, inp_lst_state_aft] =
//         [lst_state_list_bef, lst_state_list_aft].map(|l| l.0[inp_lst_state_idx].into_lst_state());
//     let [out_lst_state_bef, out_lst_state_aft] =
//         [lst_state_list_bef, lst_state_list_aft].map(|l| l.0[out_lst_state_idx].into_lst_state());

//     let inp_sol_val_delta =
//         i128::from(inp_lst_state_aft.sol_value) - i128::from(inp_lst_state_bef.sol_value);
//     let out_sol_val_delta =
//         i128::from(out_lst_state_aft.sol_value) - i128::from(out_lst_state_bef.sol_value);

//     let total_delta = inp_sol_val_delta + out_sol_val_delta;

//     let pool_sol_val_delta =
//         i128::from(pool_aft.total_sol_value) - i128::from(pool_bef.total_sol_value);

//     assert_eq!(total_delta, pool_sol_val_delta);
//     assert!(pool_sol_val_delta > 0);

//     let inp_lst_balance_delta =
//         u64::from_le_bytes(inp_lst_acc_bef.amount) - u64::from_le_bytes(inp_lst_acc_aft.amount);
//     let out_lst_balance_delta =
//         u64::from_le_bytes(out_lst_acc_aft.amount) - u64::from_le_bytes(out_lst_acc_bef.amount);
//     let protocol_fee_accumulator_balance_delta =
//         u64::from_le_bytes(protocol_fee_accumulator_aft.amount)
//             - u64::from_le_bytes(protocol_fee_accumulator_bef.amount);

//     assert_eq!(inp_lst_balance_delta, quote.0.inp);
//     assert_eq!(out_lst_balance_delta, quote.0.out);
//     assert_eq!(protocol_fee_accumulator_balance_delta, quote.0.protocol_fee);

//     Ok(())
// }

// /// To give us an upper bound on sol value of stake pools
// /// that have exchange rate > 1
// const MAX_LAMPORTS_OVER_SUPPLY: u64 = 1_000_000_000;

// proptest! {
//     #[test]
//     fn swap_exact_in_spl_multi_any(
//         (pool, inp_stake_pool_addr, inp_stake_pool, inp_lsd, out_stake_pool_addr, out_stake_pool, out_lsd, exact_in_amount) in
//             (
//                 any_pool_state(GenPoolStateArgs {
//                     bools: PoolStateBools::normal(),
//                     pks: PoolStatePks(
//                         NewPoolStatePksBuilder::start()
//                             .with_admin(None)
//                             .with_rebalance_authority(None)
//                             .with_protocol_fee_beneficiary(None)
//                             .with_pricing_program(Some(Just(inf1_pp_flatslab_core::ID).boxed()))
//                             .with_lp_token_mint(None)
//                             .build()
//                             .0
//                     ),
//                     u16s: PoolStateU16s::normal(),
//                     ..Default::default()
//                 }),
//                 any_normal_pk(),
//                 any::<u64>(),
//                 any_normal_pk(),
//                 any::<u64>(),
//             ).prop_flat_map(
//                 |(pool, inp_mint_addr, inp_spl_lamports, out_mint_addr, out_spl_lamports)| (
//                     Just(pool),
//                     any_normal_pk().prop_filter("cannot be eq mint_addr", move |x| *x != inp_mint_addr),
//                     any_spl_stake_pool(GenStakePoolArgs {
//                         pool_mint: Some(Just(inp_mint_addr).boxed()),
//                         u64s: SplStakePoolU64s(NewSplStakePoolU64sBuilder::start()
//                             .with_last_update_epoch(Just(0).boxed()) // mollusk clock defaults to epoch 0
//                             .with_total_lamports(Just(inp_spl_lamports).boxed())
//                             .with_pool_token_supply((inp_spl_lamports / MAX_LAMPORTS_OVER_SUPPLY..=u64::MAX).boxed())
//                             .build().0.map(Some)),
//                         ..Default::default()
//                     }),
//                     any_lst_state(GenLstStateArgs {
//                         sol_value:Some((0..=pool.total_sol_value / 2).boxed()),
//                         pks: LstStatePks(NewLstStatePksBuilder::start()
//                             .with_mint(inp_mint_addr)
//                             .with_sol_value_calculator(sanctum_spl_multi::ID)
//                             .build().0.map(|x| Some(Just(x).boxed()))),
//                         ..Default::default()
//                     }),
//                     any_normal_pk().prop_filter("cannot be eq mint_addr", move |x| *x != out_mint_addr),
//                     any_spl_stake_pool(GenStakePoolArgs {
//                         pool_mint: Some(Just(out_mint_addr).boxed()),
//                         u64s: SplStakePoolU64s(NewSplStakePoolU64sBuilder::start()
//                             .with_last_update_epoch(Just(0).boxed()) // mollusk clock defaults to epoch 0
//                             .with_total_lamports(Just(out_spl_lamports).boxed())
//                             .with_pool_token_supply((out_spl_lamports / MAX_LAMPORTS_OVER_SUPPLY..=u64::MAX).boxed())
//                             .build().0.map(Some)),
//                         ..Default::default()
//                     }),
//                     any_lst_state(GenLstStateArgs {
//                         sol_value:Some((0..=pool.total_sol_value / 2).boxed()),
//                         pks: LstStatePks(NewLstStatePksBuilder::start()
//                             .with_mint(out_mint_addr)
//                             .with_sol_value_calculator(sanctum_spl_multi::ID)
//                             .build().0.map(|x| Some(Just(x).boxed()))),
//                         ..Default::default()
//                     }),
//                     100..1_000_000 as u64
//                 )

//             ),

//         lsl in any_lst_state_list(Default::default(), 0..=MAX_LST_STATES),
//     ) {
//         swap_exact_in_spl_multi_proptest(
//             pool,
//             lsl,
//             inp_lsd,
//             inp_stake_pool_addr,
//             inp_stake_pool,
//             out_lsd,
//             out_stake_pool_addr,
//             out_stake_pool,
//             exact_in_amount,
//         )
//         .unwrap();
//     }
// }
