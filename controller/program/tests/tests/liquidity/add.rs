use expect_test::{expect, Expect};
use inf1_core::instructions::liquidity::add::{
    add_liquidity_ix_is_signer, add_liquidity_ix_is_writer, add_liquidity_ix_keys_owned,
    AddLiquidityIxAccs,
};
use inf1_ctl_jiminy::{
    accounts::{lst_state_list::LstStatePackedList, pool_state::PoolStatePacked},
    instructions::liquidity::{
        add::{AddLiquidityIxData, AddLiquidityIxPreKeysOwned, NewAddLiquidityIxPreAccsBuilder},
        IxArgs,
    },
    keys::{LST_STATE_LIST_ID, POOL_STATE_ID},
    ID,
};
use inf1_svc_jiminy::traits::SolValCalcAccs;

use inf1_pp_jiminy::traits::{deprecated::PriceLpTokensToMintAccs, main::PriceExactInAccs};
use inf1_std::inf1_pp_ag_std::instructions::PriceLpTokensToMintAccsAg;
use inf1_std::inf1_pp_ag_std::PricingAgTy;
use inf1_svc_ag_core::{
    inf1_svc_lido_core::solido_legacy_core::TOKENKEG_PROGRAM, instructions::SvcCalcAccsAg, SvcAgTy,
};
use inf1_test_utils::{
    acc_bef_aft, find_pool_reserves_ata, fixtures_accounts_opt_cloned,
    keys_signer_writable_to_metas, KeyedUiAccount, PkAccountTup, JUPSOL_FIXTURE_LST_IDX,
    JUPSOL_MINT,
};
use mollusk_svm::result::{InstructionResult, ProgramResult};
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
    println!(
        "Protocol pown {:#?}",
        Pubkey::new_from_array(protocol_fee_accumulator)
    );

    println!(" mint {:#?}", Pubkey::new_from_array(lst_mint));
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
        jupsol_fixtures_svc_suf().as_ref_const().suf_len(),
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

    assert_correct_sync_snapshot(
        &accounts,
        &resulting_accounts,
        JUPSOL_MINT.as_array(),
        expect!["547883064449"],
    );
}
