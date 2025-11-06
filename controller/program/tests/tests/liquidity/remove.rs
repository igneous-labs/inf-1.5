#![allow(deprecated)]

use expect_test::{expect, Expect};
use inf1_core::instructions::liquidity::remove::RemoveLiquidityIxAccs;
use inf1_ctl_jiminy::{
    accounts::{
        lst_state_list::LstStatePackedList,
        pool_state::{PoolState, PoolStatePacked},
    },
    err::Inf1CtlErr,
    instructions::liquidity::{
        remove::{
            NewRemoveLiquidityIxPreAccsBuilder, RemoveLiquidityIxData,
            RemoveLiquidityIxPreKeysOwned,
        },
        IxArgs,
    },
    keys::{LST_STATE_LIST_ID, POOL_STATE_ID},
    program_err::Inf1CtlCustomProgErr,
    ID,
};
use inf1_pp_core::pair::Pair;
use inf1_pp_flatslab_std::FlatSlabPricing;
use inf1_std::{
    quote::{
        liquidity::remove::{quote_remove_liq, RemoveLiqQuoteArgs},
        Quote,
    },
    sync::SyncSolVal,
};
use inf1_svc_jiminy::traits::{SolValCalc, SolValCalcAccs};

use inf1_std::{
    inf1_pp_ag_std::instructions::PriceLpTokensToRedeemAccsAg,
    instructions::liquidity::remove::remove_liquidity_ix_keys_owned,
};
use inf1_std::{
    inf1_pp_ag_std::PricingAgTy,
    instructions::liquidity::remove::{
        remove_liquidity_ix_is_signer, remove_liquidity_ix_is_writer,
    },
};
use inf1_svc_ag_core::{
    inf1_svc_lido_core::solido_legacy_core::TOKENKEG_PROGRAM,
    inf1_svc_spl_core::{calc::SplCalc, sanctum_spl_stake_pool_core::StakePool},
    instructions::SvcCalcAccsAg,
    SvcAgTy,
};
use inf1_test_utils::{
    acc_bef_aft, assert_jiminy_prog_err, find_pool_reserves_ata, fixtures_accounts_opt_cloned,
    keys_signer_writable_to_metas, pool_state_account, upsert_account, KeyedUiAccount,
    PkAccountTup, JUPSOL_FIXTURE_LST_IDX, JUPSOL_MINT,
};
use jiminy_cpi::program_error::INVALID_ACCOUNT_DATA;
use mollusk_svm::result::{InstructionResult, ProgramResult};
use sanctum_spl_token_jiminy::sanctum_spl_token_core::state::{
    account::{RawTokenAccount, TokenAccount},
    mint::{Mint, RawMint},
};
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

use crate::common::{
    flat_slab_pricing_lp_tokens_to_redeem_fixture_suf, jupsol_fixtures_svc_suf, SVM,
};

type RemoveLiquidityValueKeysBuilder = RemoveLiquidityIxAccs<
    [u8; 32],
    RemoveLiquidityIxPreKeysOwned,
    SvcCalcAccsAg,
    PriceLpTokensToRedeemAccsAg,
>;

#[allow(clippy::too_many_arguments)]
fn remove_liquidity_ix_pre_keys_owned(
    token_program: &[u8; 32],
    lst_mint: [u8; 32],
    lp_mint: [u8; 32],
    signer: [u8; 32],
    lst_acc: [u8; 32],
    lp_acc: [u8; 32],
    protocol_fee_accumulator: [u8; 32],
    lst_token_program: [u8; 32],
    lp_token_program: [u8; 32],
) -> RemoveLiquidityIxPreKeysOwned {
    NewRemoveLiquidityIxPreAccsBuilder::start()
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

fn remove_liquidity_ix(
    builder: &RemoveLiquidityValueKeysBuilder,
    lst_idx: u32,
    lst_value_calc_accs: u8,
    amount: u64,
    min_out: u64,
) -> Instruction {
    let accounts = keys_signer_writable_to_metas(
        remove_liquidity_ix_keys_owned(builder).seq(),
        remove_liquidity_ix_is_signer(builder).seq(),
        remove_liquidity_ix_is_writer(builder).seq(),
    );
    Instruction {
        program_id: Pubkey::new_from_array(ID),
        accounts,
        data: RemoveLiquidityIxData::new(IxArgs {
            lst_index: lst_idx,
            lst_value_calc_accs,
            amount,
            min_out,
        })
        .as_buf()
        .into(),
    }
}

fn remove_liquidity_ix_fixtures_accounts_opt(
    builder: &RemoveLiquidityValueKeysBuilder,
) -> Vec<PkAccountTup> {
    fixtures_accounts_opt_cloned(remove_liquidity_ix_keys_owned(builder).seq().copied()).collect()
}

/// Returns pool.total_sol_value delta
fn assert_correct_liq_added(
    lst_mint: &[u8; 32],
    pool_reserve_bef: &[PkAccountTup],
    pool_reserve_aft: &[PkAccountTup],
    lp_acc: [u8; 32],
    lst_acc: [u8; 32],
    expected_quote: Quote,
) -> i128 {
    let [pools, lst_state_lists, lp_acc, lst_acc] =
        [POOL_STATE_ID, LST_STATE_LIST_ID, lp_acc, lst_acc].map(|a| {
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
        .position(|s| s.into_lst_state().mint == *lst_mint)
        .unwrap();
    let [lst_state_bef, lst_state_aft] =
        [lst_state_list_bef, lst_state_list_aft].map(|l| l.0[lst_state_i].into_lst_state());

    assert_eq!(lst_state_bef.mint, *lst_mint);
    assert_eq!(lst_state_bef.mint, lst_state_aft.mint);

    assert!(lst_state_bef.sol_value < lst_state_aft.sol_value);
    assert!(pool_bef.total_sol_value < pool_aft.total_sol_value);

    let [lst_bef_balance, lst_aft_balance, lp_bef_balance, lp_aft_balance] = [
        &lst_acc[0].data,
        &lst_acc[1].data,
        &lp_acc[0].data,
        &lp_acc[1].data,
    ]
    .map(|data| {
        RawTokenAccount::of_acc_data(data)
            .and_then(TokenAccount::try_from_raw)
            .map(|a| a.amount())
            .unwrap()
    });

    // LP should be minted to acc
    assert!(lp_aft_balance < lp_bef_balance);

    // LST should be substracted from acc
    assert!(lst_bef_balance < lst_aft_balance);

    let lp_acc_balance_diff = lp_bef_balance.checked_sub(lp_aft_balance).unwrap();
    let lst_acc_balance_diff = lst_aft_balance.checked_sub(lst_bef_balance).unwrap();

    assert_eq!(expected_quote.inp, lp_acc_balance_diff);
    assert_eq!(expected_quote.out, lst_acc_balance_diff);

    i128::from(pool_aft.total_sol_value) - i128::from(pool_bef.total_sol_value)
}

fn assert_correct_sync_snapshot(
    bef: &[PkAccountTup],
    aft: &[PkAccountTup],
    lp_mint: &[u8; 32],
    lp_acc: &[u8; 32],
    lst_acc: &[u8; 32],
    expected_quote: Quote,
    expected_sol_value_delta: Expect,
) {
    let sol_delta = assert_correct_liq_added(lp_mint, bef, aft, *lp_acc, *lst_acc, expected_quote);

    expected_sol_value_delta.assert_eq(&sol_delta.to_string());
}

#[test]
fn remove_liquidity_jupsol_fixture() {
    let (jup_pf_acc_pubkey, _) =
        KeyedUiAccount::from_test_fixtures_json("jupsol-pf-accum.json").into_keyed_account();

    let (inf_token_acc_owner_pk, _) =
        KeyedUiAccount::from_test_fixtures_json("inf-token-acc-owner.json").into_keyed_account();

    let (jupsol_lst_acc_pk, _) =
        KeyedUiAccount::from_test_fixtures_json("jupsol-token-acc.json").into_keyed_account();

    let (inf_lst_acc_pk, _) =
        KeyedUiAccount::from_test_fixtures_json("inf-token-acc.json").into_keyed_account();

    let (inf_mint, inf_acc) =
        KeyedUiAccount::from_test_fixtures_json("inf-mint.json").into_keyed_account();

    let (_, jupsol_pool_acc) =
        KeyedUiAccount::from_test_fixtures_json("jupsol-pool.json").into_keyed_account();
    let (_, slab_acc) =
        KeyedUiAccount::from_test_fixtures_json("flatslab-slab.json").into_keyed_account();

    let ix_prefix = remove_liquidity_ix_pre_keys_owned(
        &TOKENKEG_PROGRAM,
        JUPSOL_MINT.to_bytes(),
        inf_mint.to_bytes(),
        inf_token_acc_owner_pk.to_bytes(),
        jupsol_lst_acc_pk.to_bytes(),
        inf_lst_acc_pk.to_bytes(),
        jup_pf_acc_pubkey.to_bytes(),
        TOKENKEG_PROGRAM,
        TOKENKEG_PROGRAM,
    );

    let builder = RemoveLiquidityValueKeysBuilder {
        ix_prefix,
        lst_calc_prog: *SvcAgTy::SanctumSplMulti(()).svc_program_id(),
        lst_calc: jupsol_fixtures_svc_suf(),
        pricing_prog: *PricingAgTy::FlatSlab(()).program_id(),
        pricing: flat_slab_pricing_lp_tokens_to_redeem_fixture_suf(),
    };

    let ix = remove_liquidity_ix(
        &builder,
        JUPSOL_FIXTURE_LST_IDX as u32,
        jupsol_fixtures_svc_suf().as_ref_const().suf_len() + 1,
        1000,
        131,
    );

    let accounts = remove_liquidity_ix_fixtures_accounts_opt(&builder);

    let InstructionResult {
        program_result,
        resulting_accounts,
        ..
    } = SVM.with(|svm| svm.process_instruction(&ix, &accounts));

    assert_eq!(program_result, ProgramResult::Success);

    let jupsol_stakepool = StakePool::borsh_de(jupsol_pool_acc.data.as_slice()).unwrap();

    let out_calc = SplCalc::new(&jupsol_stakepool, 0);
    let pricing = FlatSlabPricing::new(slab_acc.data.into_boxed_slice())
        .flat_slab_swap_pricing_for(&Pair {
            inp: &inf_mint.to_bytes(),
            out: &JUPSOL_MINT.to_bytes(),
        })
        .unwrap();

    let lp_token_supply = RawMint::of_acc_data(&inf_acc.data)
        .and_then(Mint::try_from_raw)
        .map(|a| a.supply())
        .ok_or(INVALID_ACCOUNT_DATA)
        .unwrap();

    let [pool_acc, lst_state_lists] = [POOL_STATE_ID, LST_STATE_LIST_ID]
        .map(|a| acc_bef_aft(&Pubkey::new_from_array(a), &accounts, &resulting_accounts));

    // To get the balance of the pool's LST reserve, retrieve the token account at the
    // pool reserve's address. That account is identified by checking its mint and owner.
    // Here, pool_acc[0] is the PoolState account, not the reserve.
    // Use find_pool_reserves_ata to find the Pubkey, then find that account in resulting_accounts.

    let pool_reserve_pk = find_pool_reserves_ata(&TOKENKEG_PROGRAM, &JUPSOL_MINT.to_bytes()).0;
    let pool_reserve_acc = accounts
        .iter()
        .find(|a| a.0 == pool_reserve_pk)
        .expect("pool reserve account not found");

    let lst_balance = RawTokenAccount::of_acc_data(&pool_reserve_acc.1.data)
        .and_then(TokenAccount::try_from_raw)
        .map(|a| a.amount())
        .unwrap();

    let lst_sol_val = *out_calc.lst_to_sol(lst_balance).unwrap().start();

    let list = LstStatePackedList::of_acc_data(&lst_state_lists[0].data).unwrap();

    let lst_state = list.0.get(JUPSOL_FIXTURE_LST_IDX).unwrap();
    let lst_state = unsafe { lst_state.as_lst_state() };

    let pool = unsafe { PoolState::of_acc_data(&pool_acc[0].data) }.unwrap();

    let pool_total_sol = SyncSolVal {
        pool_total: pool.total_sol_value,
        lst_old: lst_state.sol_value,
        lst_new: lst_sol_val,
    }
    .exec_checked()
    .unwrap();

    #[allow(deprecated)]
    let add_liquidity_quote_expected = quote_remove_liq(RemoveLiqQuoteArgs {
        amt: 1000,
        lp_token_supply,
        pool_total_sol_value: pool_total_sol,
        out_reserves: lst_balance,
        lp_protocol_fee_bps: pool.lp_protocol_fee_bps,
        out_mint: JUPSOL_MINT.to_bytes(),
        lp_mint: inf_mint.to_bytes(),
        out_calc,
        pricing,
    })
    .unwrap();

    #[allow(deprecated)]
    assert_correct_sync_snapshot(
        &accounts,
        &resulting_accounts,
        JUPSOL_MINT.as_array(),
        &inf_lst_acc_pk.to_bytes(),
        &jupsol_lst_acc_pk.to_bytes(),
        add_liquidity_quote_expected.0,
        expect!["547883063123"],
    );
}

#[test]
fn remove_liquidity_pool_disabled_fixture() {
    let (jup_pf_acc_pubkey, _) =
        KeyedUiAccount::from_test_fixtures_json("jupsol-pf-accum.json").into_keyed_account();

    let (inf_token_acc_owner_pk, _) =
        KeyedUiAccount::from_test_fixtures_json("inf-token-acc-owner.json").into_keyed_account();

    let (jupsol_lst_acc_pk, _) =
        KeyedUiAccount::from_test_fixtures_json("jupsol-token-acc.json").into_keyed_account();

    let (inf_lst_acc_pk, _) =
        KeyedUiAccount::from_test_fixtures_json("inf-token-acc.json").into_keyed_account();

    let (inf_mint, _) =
        KeyedUiAccount::from_test_fixtures_json("inf-mint.json").into_keyed_account();

    let (_, pool_state_acc) =
        KeyedUiAccount::from_test_fixtures_json("pool-state.json").into_keyed_account();

    let ix_prefix = remove_liquidity_ix_pre_keys_owned(
        &TOKENKEG_PROGRAM,
        JUPSOL_MINT.to_bytes(),
        inf_mint.to_bytes(),
        inf_token_acc_owner_pk.to_bytes(),
        jupsol_lst_acc_pk.to_bytes(),
        inf_lst_acc_pk.to_bytes(),
        jup_pf_acc_pubkey.to_bytes(),
        TOKENKEG_PROGRAM,
        TOKENKEG_PROGRAM,
    );

    let builder = RemoveLiquidityValueKeysBuilder {
        ix_prefix,
        lst_calc_prog: *SvcAgTy::SanctumSplMulti(()).svc_program_id(),
        lst_calc: jupsol_fixtures_svc_suf(),
        pricing_prog: *PricingAgTy::FlatSlab(()).program_id(),
        pricing: flat_slab_pricing_lp_tokens_to_redeem_fixture_suf(),
    };

    let ix = remove_liquidity_ix(
        &builder,
        JUPSOL_FIXTURE_LST_IDX as u32,
        jupsol_fixtures_svc_suf().as_ref_const().suf_len() + 1,
        1000,
        // Review this
        131,
    );

    let mut accounts = remove_liquidity_ix_fixtures_accounts_opt(&builder);

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
fn remove_liquidity_pool_rebalancing_fixture() {
    let (jup_pf_acc_pubkey, _) =
        KeyedUiAccount::from_test_fixtures_json("jupsol-pf-accum.json").into_keyed_account();

    let (inf_token_acc_owner_pk, _) =
        KeyedUiAccount::from_test_fixtures_json("inf-token-acc-owner.json").into_keyed_account();

    let (jupsol_lst_acc_pk, _) =
        KeyedUiAccount::from_test_fixtures_json("jupsol-token-acc.json").into_keyed_account();

    let (inf_lst_acc_pk, _) =
        KeyedUiAccount::from_test_fixtures_json("inf-token-acc.json").into_keyed_account();

    let (inf_mint, _) =
        KeyedUiAccount::from_test_fixtures_json("inf-mint.json").into_keyed_account();

    let (_, pool_state_acc) =
        KeyedUiAccount::from_test_fixtures_json("pool-state.json").into_keyed_account();

    let ix_prefix = remove_liquidity_ix_pre_keys_owned(
        &TOKENKEG_PROGRAM,
        JUPSOL_MINT.to_bytes(),
        inf_mint.to_bytes(),
        inf_token_acc_owner_pk.to_bytes(),
        jupsol_lst_acc_pk.to_bytes(),
        inf_lst_acc_pk.to_bytes(),
        jup_pf_acc_pubkey.to_bytes(),
        TOKENKEG_PROGRAM,
        TOKENKEG_PROGRAM,
    );

    let builder = RemoveLiquidityValueKeysBuilder {
        ix_prefix,
        lst_calc_prog: *SvcAgTy::SanctumSplMulti(()).svc_program_id(),
        lst_calc: jupsol_fixtures_svc_suf(),
        pricing_prog: *PricingAgTy::FlatSlab(()).program_id(),
        pricing: flat_slab_pricing_lp_tokens_to_redeem_fixture_suf(),
    };

    let ix = remove_liquidity_ix(
        &builder,
        JUPSOL_FIXTURE_LST_IDX as u32,
        jupsol_fixtures_svc_suf().as_ref_const().suf_len() + 1,
        1000,
        // Review this
        131,
    );

    let mut accounts = remove_liquidity_ix_fixtures_accounts_opt(&builder);

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
