use expect_test::{expect, Expect};
#[allow(deprecated)]
use inf1_core::instructions::liquidity::add::AddLiquidityIxAccs;
use inf1_ctl_jiminy::{
    accounts::{
        lst_state_list::{LstStatePackedList, LstStatePackedListMut},
        pool_state::{PoolState, PoolStatePacked},
    },
    err::Inf1CtlErr,
    instructions::{
        liquidity::{
            add::{
                AddLiquidityIxData, AddLiquidityIxPreKeysOwned, NewAddLiquidityIxPreAccsBuilder,
            },
            IxArgs,
        },
        sync_sol_value::NewSyncSolValueIxPreAccsBuilder,
    },
    keys::{LST_STATE_LIST_ID, POOL_STATE_ID},
    program_err::Inf1CtlCustomProgErr,
    ID,
};
use inf1_pp_core::{
    instructions::{
        deprecated::lp::mint::PriceLpTokensToMintIxArgs, price::exact_in::PriceExactInIxArgs,
    },
    pair::Pair,
    traits::{deprecated::PriceLpTokensToMint, main::PriceExactIn},
};
use inf1_pp_flatslab_std::FlatSlabPricing;
use inf1_std::{
    instructions::sync_sol_value::SyncSolValueIxAccs,
    quote::liquidity::add::{quote_add_liq, AddLiqQuoteArgs},
    sync::SyncSolVal,
    InfStd,
};
use inf1_svc_jiminy::traits::{SolValCalc, SolValCalcAccs};

use inf1_std::{
    inf1_pp_ag_std::instructions::PriceLpTokensToMintAccsAg,
    instructions::liquidity::add::add_liquidity_ix_keys_owned,
};
use inf1_std::{
    inf1_pp_ag_std::{PricingAgTy, PricingProgAg},
    instructions::liquidity::add::{add_liquidity_ix_is_signer, add_liquidity_ix_is_writer},
};
use inf1_svc_ag_core::{
    inf1_svc_lido_core::solido_legacy_core::TOKENKEG_PROGRAM,
    inf1_svc_spl_core::{calc::SplCalc, sanctum_spl_stake_pool_core::StakePool},
    instructions::SvcCalcAccsAg,
    SvcAgTy,
};
use inf1_test_utils::{
    acc_bef_aft, assert_jiminy_prog_err, find_pool_reserves_ata, fixtures_accounts_opt_cloned,
    keys_signer_writable_to_metas, lst_state_list_account, pool_state_account, upsert_account,
    KeyedUiAccount, PkAccountTup, JUPSOL_FIXTURE_LST_IDX, JUPSOL_MINT,
};
use jiminy_cpi::program_error::INVALID_ACCOUNT_DATA;
use mollusk_svm::result::{InstructionResult, ProgramResult};
use sanctum_spl_token_jiminy::sanctum_spl_token_core::state::{
    account::{RawTokenAccount, TokenAccount},
    mint::{Mint, RawMint},
};
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

use crate::common::{flat_slab_pricing_fixture_suf, jupsol_fixtures_svc_suf, SVM};

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
    expected_sol_value_delta: Expect,
) {
    let sol_delta = assert_correct_liq_added(lp_mint, bef, aft);

    expected_sol_value_delta.assert_eq(&sol_delta.to_string());
}

#[test]
fn add_liquidity_jupsol_fixture() {
    let (jup_pf_acc_pubkey, _) =
        KeyedUiAccount::from_test_fixtures_json("jupsol-pf-accum.json").into_keyed_account();

    let (jupsol_token_acc_owner_pk, _) =
        KeyedUiAccount::from_test_fixtures_json("jupsol-token-acc-owner.json").into_keyed_account();

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

    let ix_prefix = add_liquidity_ix_pre_keys_owned(
        &TOKENKEG_PROGRAM,
        JUPSOL_MINT.to_bytes(),
        inf_mint.to_bytes(),
        jupsol_token_acc_owner_pk.to_bytes(),
        jupsol_lst_acc_pk.to_bytes(),
        inf_lst_acc_pk.to_bytes(),
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
        jupsol_fixtures_svc_suf().as_ref_const().suf_len() + 1,
        1000,
        // Review this
        131,
    );

    let accounts = add_liquidity_ix_fixtures_accounts_opt(&builder);

    let InstructionResult {
        program_result,
        resulting_accounts,
        ..
    } = SVM.with(|svm| svm.process_instruction(&ix, &accounts));

    assert_eq!(program_result, ProgramResult::Success);

    let jupsol_stakepool = StakePool::borsh_de(jupsol_pool_acc.data.as_slice()).unwrap();

    let inp_calc = SplCalc::new(&jupsol_stakepool, 0);
    let pricing = FlatSlabPricing::new(slab_acc.data.into_boxed_slice())
        .flat_slab_swap_pricing_for(&Pair {
            inp: &JUPSOL_MINT.to_bytes(),
            out: &inf_mint.to_bytes(),
        })
        .unwrap();

    let lp_token_supply = RawMint::of_acc_data(&inf_acc.data)
        .and_then(Mint::try_from_raw)
        .map(|a| a.supply())
        .ok_or(INVALID_ACCOUNT_DATA)
        .unwrap();

    let [pool_acc, lp_acc, lst_acc, lst_state_lists] = [
        POOL_STATE_ID,
        inf_lst_acc_pk.to_bytes(),
        jupsol_lst_acc_pk.to_bytes(),
        LST_STATE_LIST_ID,
    ]
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

    let lst_sol_val = *inp_calc.lst_to_sol(lst_balance).unwrap().start();

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

    let amt_sol_val = *inp_calc.lst_to_sol(1000).unwrap().start();

    let r = pricing.price_exact_in(PriceExactInIxArgs {
        sol_value: amt_sol_val,
        amt: 1000,
    });

    println!("amt_sol_val{:?}", amt_sol_val);
    println!("Pricing r{:?}", r.unwrap());
    println!(
        "ps {:?}",
        (r.unwrap() * lp_token_supply) / pool.total_sol_value
    );

    println!("{:?}", lp_token_supply);
    println!("pool.total_sol_value{:?} 1", pool.total_sol_value);
    println!("pool.total_sol_value{:?}", pool_total_sol);
    println!("JUPSOL_MINT.to_bytes() {:?}", JUPSOL_MINT.to_bytes());
    //TODO(pavs) call sync sol value

    #[allow(deprecated)]
    let add_liquidity_quote_expected = quote_add_liq(AddLiqQuoteArgs {
        amt: 1000,
        lp_token_supply,
        lp_mint: pool.lp_token_mint,
        lp_protocol_fee_bps: pool.lp_protocol_fee_bps,
        pool_total_sol_value: pool_total_sol,
        inp_calc,
        pricing,
        inp_mint: JUPSOL_MINT.to_bytes(),
    })
    .unwrap();

    let lst_bef_balance = RawTokenAccount::of_acc_data(&lst_acc[0].data)
        .and_then(TokenAccount::try_from_raw)
        .map(|a| a.amount())
        .unwrap();

    let lst_aft_balance = RawTokenAccount::of_acc_data(&lst_acc[1].data)
        .and_then(TokenAccount::try_from_raw)
        .map(|a| a.amount())
        .unwrap();

    let lp_bef_balance = RawTokenAccount::of_acc_data(&lp_acc[0].data)
        .and_then(TokenAccount::try_from_raw)
        .map(|a| a.amount())
        .unwrap();

    let lp_aft_balance = RawTokenAccount::of_acc_data(&lp_acc[1].data)
        .and_then(TokenAccount::try_from_raw)
        .map(|a| a.amount())
        .unwrap();

    // LP should be minted to acc
    assert!(lp_aft_balance > lp_bef_balance);

    // LST should be substracted from acc
    assert!(lst_bef_balance > lst_aft_balance);

    let lp_acc_balance_diff = lp_aft_balance.checked_sub(lp_bef_balance).unwrap();
    let lst_acc_balance_diff = lst_bef_balance.checked_sub(lst_aft_balance).unwrap();

    assert_eq!(add_liquidity_quote_expected.0.out, lp_acc_balance_diff);

    assert_correct_sync_snapshot(
        &accounts,
        &resulting_accounts,
        JUPSOL_MINT.as_array(),
        expect!["547883065552"],
    );
}

#[test]
fn add_liquidity_input_disabled_fixture() {
    let (jup_pf_acc_pubkey, _) =
        KeyedUiAccount::from_test_fixtures_json("jupsol-pf-accum.json").into_keyed_account();

    let (jupsol_token_acc_owner_pk, _) =
        KeyedUiAccount::from_test_fixtures_json("jupsol-token-acc-owner.json").into_keyed_account();

    let (jupsol_lst_acc_pk, _) =
        KeyedUiAccount::from_test_fixtures_json("jupsol-token-acc.json").into_keyed_account();

    let (inf_lst_acc_pk, _) =
        KeyedUiAccount::from_test_fixtures_json("inf-token-acc.json").into_keyed_account();

    let (inf_mint, _) =
        KeyedUiAccount::from_test_fixtures_json("inf-mint.json").into_keyed_account();

    let (_, mut lst_state_list_acc) =
        KeyedUiAccount::from_test_fixtures_json("lst-state-list.json").into_keyed_account();

    let ix_prefix = add_liquidity_ix_pre_keys_owned(
        &TOKENKEG_PROGRAM,
        JUPSOL_MINT.to_bytes(),
        inf_mint.to_bytes(),
        jupsol_token_acc_owner_pk.to_bytes(),
        jupsol_lst_acc_pk.to_bytes(),
        inf_lst_acc_pk.to_bytes(),
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
        jupsol_fixtures_svc_suf().as_ref_const().suf_len() + 1,
        1000,
        // Review this
        131,
    );

    let mut accounts = add_liquidity_ix_fixtures_accounts_opt(&builder);

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
fn add_liquidity_pool_disabled_fixture() {
    let (jup_pf_acc_pubkey, _) =
        KeyedUiAccount::from_test_fixtures_json("jupsol-pf-accum.json").into_keyed_account();

    let (jupsol_token_acc_owner_pk, _) =
        KeyedUiAccount::from_test_fixtures_json("jupsol-token-acc-owner.json").into_keyed_account();

    let (jupsol_lst_acc_pk, _) =
        KeyedUiAccount::from_test_fixtures_json("jupsol-token-acc.json").into_keyed_account();

    let (inf_lst_acc_pk, _) =
        KeyedUiAccount::from_test_fixtures_json("inf-token-acc.json").into_keyed_account();

    let (inf_mint, _) =
        KeyedUiAccount::from_test_fixtures_json("inf-mint.json").into_keyed_account();

    let (_, pool_state_acc) =
        KeyedUiAccount::from_test_fixtures_json("pool-state.json").into_keyed_account();

    let ix_prefix = add_liquidity_ix_pre_keys_owned(
        &TOKENKEG_PROGRAM,
        JUPSOL_MINT.to_bytes(),
        inf_mint.to_bytes(),
        jupsol_token_acc_owner_pk.to_bytes(),
        jupsol_lst_acc_pk.to_bytes(),
        inf_lst_acc_pk.to_bytes(),
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
        jupsol_fixtures_svc_suf().as_ref_const().suf_len() + 1,
        1000,
        // Review this
        131,
    );

    let mut accounts = add_liquidity_ix_fixtures_accounts_opt(&builder);

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
fn add_liquidity_pool_rebalancing_fixture() {
    let (jup_pf_acc_pubkey, _) =
        KeyedUiAccount::from_test_fixtures_json("jupsol-pf-accum.json").into_keyed_account();

    let (jupsol_token_acc_owner_pk, _) =
        KeyedUiAccount::from_test_fixtures_json("jupsol-token-acc-owner.json").into_keyed_account();

    let (jupsol_lst_acc_pk, _) =
        KeyedUiAccount::from_test_fixtures_json("jupsol-token-acc.json").into_keyed_account();

    let (inf_lst_acc_pk, _) =
        KeyedUiAccount::from_test_fixtures_json("inf-token-acc.json").into_keyed_account();

    let (inf_mint, _) =
        KeyedUiAccount::from_test_fixtures_json("inf-mint.json").into_keyed_account();

    let (_, pool_state_acc) =
        KeyedUiAccount::from_test_fixtures_json("pool-state.json").into_keyed_account();

    let ix_prefix = add_liquidity_ix_pre_keys_owned(
        &TOKENKEG_PROGRAM,
        JUPSOL_MINT.to_bytes(),
        inf_mint.to_bytes(),
        jupsol_token_acc_owner_pk.to_bytes(),
        jupsol_lst_acc_pk.to_bytes(),
        inf_lst_acc_pk.to_bytes(),
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
        jupsol_fixtures_svc_suf().as_ref_const().suf_len() + 1,
        1000,
        // Review this
        131,
    );

    let mut accounts = add_liquidity_ix_fixtures_accounts_opt(&builder);

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
