use std::{ops::Range, str::FromStr};

use expect_test::{expect, Expect};
use inf1_core::instructions::liquidity::add::{
    add_liquidity_ix_is_signer, add_liquidity_ix_is_writer, add_liquidity_ix_keys_owned,
    AddLiquidityIxAccs,
};
use inf1_ctl_jiminy::{
    accounts::{
        lst_state_list::LstStatePackedList,
        pool_state::{PoolState, PoolStatePacked},
    },
    err::Inf1CtlErr,
    instructions::liquidity::{
        add::{AddLiquidityIxData, AddLiquidityIxPreKeysOwned, NewAddLiquidityIxPreAccsBuilder},
        IxArgs,
    },
    keys::{
        LST_STATE_LIST_ID, POOL_STATE_ID, PROTOCOL_FEE_BUMP, PROTOCOL_FEE_ID, PROTOCOL_FEE_ID_STR,
    },
    pda::const_find_protocol_fee,
    pda_onchain::create_raw_protocol_fee_accumulator_addr,
    program_err::Inf1CtlCustomProgErr,
    ID,
};
use inf1_svc_jiminy::traits::SolValCalcAccs;
use jiminy_cpi::program_error::INVALID_ARGUMENT;
use jiminy_sysvar_rent::Rent;
use proptest::prop_assert_eq;
use solana_account::Account;

use inf1_pp_jiminy::{
    instructions::price::{IxAccs, IxPreAccFlags},
    traits::{deprecated::PriceLpTokensToMintAccs, main::PriceExactInAccs},
};
use inf1_std::inf1_pp_ag_std::{
    inf1_pp_flatslab_std::instructions::pricing::FlatSlabPpAccs, PricingAgTy,
};
use inf1_std::inf1_pp_ag_std::{
    inf1_pp_flatslab_std::keys::LP_MINT_ID_STR, instructions::PriceLpTokensToMintAccsAg,
};
use inf1_svc_ag_core::{
    inf1_svc_lido_core::solido_legacy_core::{SYSTEM_PROGRAM, TOKENKEG_PROGRAM},
    inf1_svc_spl_core::{
        instructions::sol_val_calc::SanctumSplMultiCalcAccs, keys::sanctum_spl_multi,
    },
    instructions::SvcCalcAccsAg,
    SvcAgTy,
};
use inf1_test_utils::{
    acc_bef_aft, any_lst_state, any_lst_state_list, any_normal_pk, any_pool_state,
    any_spl_stake_pool, assert_jiminy_prog_err, find_pool_reserves_ata,
    fixtures_accounts_opt_cloned, keys_signer_writable_to_metas, mock_mint, mock_system_acc,
    mock_token_acc, raw_mint, raw_token_acc, silence_mollusk_logs, upsert_account, AnyLstStateArgs,
    AnyPoolStateArgs, GenStakePoolArgs, LstStateData, LstStateListData, LstStatePks,
    NewLstStatePksBuilder, NewPoolStateBoolsBuilder, NewSplStakePoolU64sBuilder, PkAccountTup,
    PoolStateBools, SplStakePoolU64s, JUPSOL_FIXTURE_LST_IDX, JUPSOL_MINT,
};
use mollusk_svm::result::{InstructionResult, ProgramResult};
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

use crate::{
    common::{
        flat_slab_pricing_fixture_suf, jupsol_fixtures_svc_suf, max_sol_val_no_overflow,
        MAX_LAMPORTS_OVER_SUPPLY, MAX_LST_STATES, SVM,
    },
    tests::set_sol_value_calculator::assert_correct_set,
    TestErrorType,
};

use proptest::{prelude::*, test_runner::TestCaseResult};

type AddLiquidityValueKeysBuilder = AddLiquidityIxAccs<
    [u8; 32],
    AddLiquidityIxPreKeysOwned,
    SvcCalcAccsAg,
    PriceLpTokensToMintAccsAg,
>;

fn add_liquidity_ix_pre_keys_owned(
    token_program: &[u8; 32],
    lst_mint: [u8; 32],
    lp_mint: [u8; 32],
    signer: [u8; 32],
    lst_acc: [u8; 32],
    lp_acc: [u8; 32],
    protocol_fee_accumulator: [u8; 32],
    lst_token_program: [u8; 32],
    lp_token_program: [u8; 32],
) -> AddLiquidityIxPreKeysOwned {
    NewAddLiquidityIxPreAccsBuilder::start()
        .with_signer(signer)
        .with_lst_mint(lst_mint)
        .with_lst_acc(lst_acc)
        .with_lp_acc(lp_acc)
        .with_lp_token_mint(lp_mint)
        .with_protocol_fee_accumulator(protocol_fee_accumulator)
        .with_lst_token_program(lst_token_program)
        .with_lp_token_program(lp_token_program)
        .with_pool_state(POOL_STATE_ID)
        .with_lst_state_list(LST_STATE_LIST_ID)
        .with_pool_reserves(
            find_pool_reserves_ata(token_program, &lst_mint)
                .0
                .to_bytes(),
        )
        .build()
}

fn add_liquidity_ix(
    builder: &AddLiquidityValueKeysBuilder,
    lst_idx: u32,
    lst_value_calc_accs: u8,
    amount: u64,
    min_out: u64,
) -> Instruction {
    let accounts = keys_signer_writable_to_metas(
        add_liquidity_ix_keys_owned(builder).seq(),
        add_liquidity_ix_is_signer(builder).seq(),
        add_liquidity_ix_is_writer(builder).seq(),
    );
    Instruction {
        program_id: Pubkey::new_from_array(ID),
        accounts,
        data: AddLiquidityIxData::new(IxArgs {
            lst_index: lst_idx,
            lst_value_calc_accs,
            amount,
            min_out,
        })
        .as_buf()
        .into(),
    }
}

fn add_liquidity_ix_fixtures_accounts_opt(
    builder: &AddLiquidityValueKeysBuilder,
) -> Vec<PkAccountTup> {
    fixtures_accounts_opt_cloned(add_liquidity_ix_keys_owned(builder).seq().copied()).collect()
}

// Check liquidity of sol in the pool increases but not sol value of LST
fn assert_correct_liq_added(
    lp_mint: &[u8; 32],
    pool_reserve_bef: &[PkAccountTup],
    pool_reserve_aft: &[PkAccountTup],
) -> i128 {
    let [pools, lst_state_lists] = [POOL_STATE_ID, LST_STATE_LIST_ID].map(|a| {
        acc_bef_aft(
            &Pubkey::new_from_array(a),
            pool_reserve_bef,
            pool_reserve_aft,
        )
    });
    let [pool_bef, pool_aft] = pools.each_ref().map(|a| {
        PoolStatePacked::of_acc_data(&a.data)
            .unwrap()
            .into_pool_state()
    });
    let [lst_state_list_bef, lst_state_list_aft] = lst_state_lists
        .each_ref()
        .map(|a| LstStatePackedList::of_acc_data(&a.data).unwrap());
    let lst_state_i = lst_state_list_bef
        .0
        .iter()
        .position(|s| s.into_lst_state().mint == *lp_mint)
        .unwrap();
    let [lst_state_bef, lst_state_aft] =
        [lst_state_list_bef, lst_state_list_aft].map(|l| l.0[lst_state_i].into_lst_state());

    assert_eq!(lst_state_bef.mint, *lp_mint);
    assert_eq!(lst_state_bef.mint, lst_state_aft.mint);
    assert!(lst_state_bef.sol_value < lst_state_aft.sol_value);
    assert!(pool_bef.total_sol_value < pool_aft.total_sol_value);

    let delta = i128::from(pool_aft.total_sol_value) - i128::from(pool_bef.total_sol_value);

    delta
}

fn assert_correct_sync_snapshot(
    bef: &[PkAccountTup],
    aft: &[PkAccountTup],
    lp_mint: &[u8; 32],
    expected_sol_val_delta: Expect,
) {
    let delta = assert_correct_liq_added(lp_mint, bef, aft);
    expected_sol_val_delta.assert_eq(&delta.to_string());
}

#[test]
fn add_liquidity_jupsol_fixture() {
    let lst_acc = Pubkey::new_unique();
    let lp_acc = Pubkey::new_unique();
    let signer = Pubkey::new_unique().to_bytes();

    let inf_pubkey = match Pubkey::from_str(LP_MINT_ID_STR) {
        Ok(pubkey) => pubkey,
        Err(_) => panic!("Cannot init inf pubkey"),
    };

    let jup_pf_acc_pubkey = match Pubkey::from_str("9twt5sCzyPvVNnd4SXmZNyA8V8QnmU3EY7XG9wGJsBRm") {
        Ok(pubkey) => pubkey,
        Err(_) => panic!("Cannot init inf pubkey"),
    };

    let ix_prefix = add_liquidity_ix_pre_keys_owned(
        &TOKENKEG_PROGRAM,
        JUPSOL_MINT.to_bytes(),
        inf_pubkey.to_bytes(),
        signer,
        lst_acc.to_bytes(),
        lp_acc.to_bytes(),
        jup_pf_acc_pubkey.to_bytes(),
        TOKENKEG_PROGRAM,
        TOKENKEG_PROGRAM,
    );

    let builder = AddLiquidityValueKeysBuilder {
        ix_prefix,
        lst_calc_prog: *SvcAgTy::SanctumSplMulti(()).svc_program_id(),
        lst_calc: jupsol_fixtures_svc_suf(),
        pricing_prog: *PricingAgTy::FlatSlab(()).program_id(),
        pricing: flat_slab_pricing_fixture_suf(),
    };

    let ix = add_liquidity_ix(
        &builder,
        JUPSOL_FIXTURE_LST_IDX as u32,
        jupsol_fixtures_svc_suf().as_ref_const().suf_len(),
        1000,
        // Review this
        131,
    );

    let mut accounts = add_liquidity_ix_fixtures_accounts_opt(&builder);
    upsert_account(
        &mut accounts,
        (Pubkey::new_from_array(signer), mock_system_acc([].to_vec())),
    );

    upsert_account(
        &mut accounts,
        (
            lst_acc,
            mock_token_acc(raw_token_acc(JUPSOL_MINT.to_bytes(), signer, 100)),
        ),
    );

    upsert_account(
        &mut accounts,
        (
            lp_acc,
            mock_token_acc(raw_token_acc(inf_pubkey.to_bytes(), signer, 100)),
        ),
    );

    upsert_account(
        &mut accounts,
        (
            inf_pubkey,
            mock_mint(raw_mint(Some(POOL_STATE_ID), None, 100000, 9)),
        ),
    );

    let InstructionResult {
        program_result,
        resulting_accounts,
        ..
    } = SVM.with(|svm| svm.process_instruction(&ix, &accounts));

    assert_eq!(program_result, ProgramResult::Success);

    assert_correct_sync_snapshot(
        &accounts,
        &resulting_accounts,
        JUPSOL_MINT.as_array(),
        expect!["547883064449"],
    );
}

fn add_liquidity_prop_test(
    mut lsl: LstStateListData,
    lsd: LstStateData,
    lst_mint: [u8; 32],
    lst_calc_prog: [u8; 32],
    lst_calc: SvcCalcAccsAg,
    pricing_prog: [u8; 32],
    pricing: PriceLpTokensToMintAccsAg,
) -> TestCaseResult {
    let lst_idx = lsl.upsert(lsd);
    let lst_acc = Pubkey::new_unique();
    let lp_acc = Pubkey::new_unique();
    let signer = Pubkey::new_unique().to_bytes();

    let inf_pubkey = match Pubkey::from_str(LP_MINT_ID_STR) {
        Ok(pubkey) => pubkey,
        Err(_) => panic!("Cannot init inf pubkey"),
    };

    let ix_prefix = add_liquidity_ix_pre_keys_owned(
        &TOKENKEG_PROGRAM,
        lst_mint,
        inf_pubkey.to_bytes(),
        signer,
        lst_acc.to_bytes(),
        lp_acc.to_bytes(),
        lsd.protocol_fee_accumulator,
        TOKENKEG_PROGRAM,
        TOKENKEG_PROGRAM,
    );

    let builder = AddLiquidityValueKeysBuilder {
        ix_prefix,
        lst_calc_prog,
        lst_calc,
        pricing_prog,
        pricing,
    };

    let ix = add_liquidity_ix(
        &builder,
        lst_idx as u32,
        lst_calc.suf_len(),
        1000, //TODO(pavs) Review this
        131,
    );

    let mut accounts = add_liquidity_ix_fixtures_accounts_opt(&builder);
    upsert_account(
        &mut accounts,
        (Pubkey::new_from_array(signer), mock_system_acc([].to_vec())),
    );

    upsert_account(
        &mut accounts,
        (
            lst_acc,
            mock_token_acc(raw_token_acc(lst_mint, signer, 100)),
        ),
    );

    upsert_account(
        &mut accounts,
        (
            lp_acc,
            mock_token_acc(raw_token_acc(inf_pubkey.to_bytes(), signer, 100)),
        ),
    );

    upsert_account(
        &mut accounts,
        (
            inf_pubkey,
            mock_mint(raw_mint(Some(POOL_STATE_ID), None, 100000, 9)),
        ),
    );

    let InstructionResult {
        program_result,
        resulting_accounts,
        ..
    } = SVM.with(|svm| svm.process_instruction(&ix, &accounts));

    prop_assert_eq!(program_result, ProgramResult::Success);
    assert_correct_set(&accounts, &resulting_accounts, &lst_mint, &lst_calc_prog);

    Ok(())
}

proptest! {
    #[test]
    fn add_liquidity_any(
        (pool, lsd, stake_pool_addr, stake_pool) in
        (any_pool_state(AnyPoolStateArgs {
            bools: PoolStateBools(NewPoolStateBoolsBuilder::start()
                .with_is_disabled(false)
                .with_is_rebalancing(true)
                .build().0.map(|x| Some(Just(x).boxed()))),
            ..Default::default()
        }),
        any_normal_pk(),
        any::<u64>(),
        ).prop_flat_map(
            |(pool, mint_addr, spl_lamports)| (
                Just(pool),
                any_normal_pk().prop_filter("cannot be eq mint_addr", move |x| *x != mint_addr),
                any_spl_stake_pool(GenStakePoolArgs {
                    pool_mint: Some(Just(mint_addr).boxed()),
                    u64s: SplStakePoolU64s(NewSplStakePoolU64sBuilder::start()
                        .with_last_update_epoch(Just(0).boxed())
                        .with_total_lamports(Just(spl_lamports).boxed())
                        .with_pool_token_supply((spl_lamports / MAX_LAMPORTS_OVER_SUPPLY..=u64::MAX).boxed())
                        .build().0.map(Some)),
                    ..Default::default()
                }),
                any_lst_state(
                    AnyLstStateArgs {
                        sol_value: Some((0..=pool.total_sol_value).boxed()),
                        pks: LstStatePks(NewLstStatePksBuilder::start()
                            .with_mint(mint_addr)
                            .with_sol_value_calculator(sanctum_spl_multi::ID)
                            .build().0.map(|x| Some(Just(x).boxed()))),
                        ..Default::default()
                    },
                    None,
                ),
            )
        ).prop_flat_map(
            |(pool, stake_pool_addr, stake_pool, lsd)| (
                Just(pool),
                Just(lsd),
                Just(stake_pool_addr),
                Just(stake_pool),
            )
        ),
        lsl in any_lst_state_list(Default::default(), None, 0..=MAX_LST_STATES),
    ) {
        add_liquidity_prop_test(
            lsl,
            lsd,
            lsd.lst_state.mint,
            *SvcAgTy::SanctumSplMulti(()).svc_program_id(),
            SvcCalcAccsAg::SanctumSplMulti(SanctumSplMultiCalcAccs { stake_pool_addr }),
            *PricingAgTy::FlatSlab(()).program_id(),
            PriceLpTokensToMintAccsAg::FlatSlab(FlatSlabPpAccs::MAINNET)
        ).unwrap();
    }
}
