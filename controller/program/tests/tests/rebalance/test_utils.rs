use std::collections::HashMap;

use inf1_core::instructions::rebalance::start::StartRebalanceIxAccs;
use inf1_ctl_jiminy::{
    accounts::{
        lst_state_list::LstStatePackedList,
        pool_state::{PoolState, PoolStatePacked},
        rebalance_record::RebalanceRecord,
    },
    instructions::rebalance::{
        end::EndRebalanceIxData,
        start::{
            NewStartRebalanceIxPreAccsBuilder, StartRebalanceIxData, StartRebalanceIxPreKeysOwned,
        },
    },
    keys::{INSTRUCTIONS_SYSVAR_ID, LST_STATE_LIST_ID, POOL_STATE_ID, REBALANCE_RECORD_ID},
    ID,
};
use inf1_std::instructions::rebalance::{end::EndRebalanceIxAccs, start::StartRebalanceIxArgs};
use inf1_svc_ag_core::{
    inf1_svc_lido_core::solido_legacy_core::TOKENKEG_PROGRAM,
    inf1_svc_wsol_core::instructions::sol_val_calc::WsolCalcAccs, instructions::SvcCalcAccsAg,
    SvcAgTy,
};
use inf1_test_utils::{
    acc_bef_aft, assert_diffs_lst_state_list, assert_diffs_pool_state, gen_lst_state,
    keys_signer_writable_to_metas, lst_state_list_account, mock_mint, mock_token_acc,
    pool_state_account, raw_mint, raw_token_acc, u8_to_bool, upsert_account, Diff,
    DiffLstStateArgs, DiffsPoolStateArgs, GenLstStateArgs, LstStateData, LstStateListChanges,
    LstStateListData, NewLstStateBumpsBuilder, NewLstStatePksBuilder, PkAccountTup, PoolStateBools,
    ALL_FIXTURES, JUPSOL_FIXTURE_LST_IDX, WSOL_MINT,
};
use sanctum_system_jiminy::sanctum_system_core::ID as SYSTEM_PROGRAM_ID;
use solana_account::Account;
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

use crate::common::jupsol_fixtures_svc_suf;

pub fn assert_start_success(
    bef: &[PkAccountTup],
    aft: &[PkAccountTup],
    out_mint: &[u8; 32],
    inp_mint: &[u8; 32],
) {
    let [pool_accounts, lst_state_accounts] = [POOL_STATE_ID, LST_STATE_LIST_ID]
        .map(|pk| acc_bef_aft(&Pubkey::new_from_array(pk), bef, aft));

    let [pool_bef, pool_aft] = pool_accounts.each_ref().map(|acc| {
        PoolStatePacked::of_acc_data(&acc.data)
            .expect("pool packed (before/after)")
            .into_pool_state()
    });

    assert_diffs_pool_state(
        &DiffsPoolStateArgs {
            bools: PoolStateBools::default().with_is_rebalancing(Diff::StrictChanged(false, true)),
            total_sol_value: Diff::Pass,
            ..Default::default()
        },
        &pool_bef,
        &pool_aft,
    );

    let [lst_state_list_bef, lst_state_list_aft]: [Vec<_>; 2] =
        lst_state_accounts.each_ref().map(|acc| {
            LstStatePackedList::of_acc_data(&acc.data)
                .expect("lst state list packed")
                .0
                .iter()
                .map(|packed| packed.into_lst_state())
                .collect()
        });

    let diffs = LstStateListChanges::new(&lst_state_list_bef)
        .with_diff_by_mint(
            out_mint,
            DiffLstStateArgs {
                sol_value: Diff::Pass,
                ..Default::default()
            },
        )
        .with_diff_by_mint(
            inp_mint,
            DiffLstStateArgs {
                sol_value: Diff::Pass,
                ..Default::default()
            },
        )
        .build();
    assert_diffs_lst_state_list(&diffs, &lst_state_list_bef, &lst_state_list_aft);

    let inp_lst_idx = lst_state_list_bef
        .iter()
        .position(|state| state.mint == *inp_mint)
        .expect("input LST exists");

    let rr_pk = Pubkey::new_from_array(REBALANCE_RECORD_ID);
    let rr_acc = aft
        .iter()
        .find_map(|(pk, acc)| (*pk == rr_pk).then_some(acc))
        .expect("rebalance record exists");

    let rr =
        unsafe { RebalanceRecord::of_acc_data(&rr_acc.data).expect("rebalance record unpack") };

    assert_eq!(
        rr.inp_lst_index, inp_lst_idx as u32,
        "dst lst index mismatch"
    );
    assert!(
        rr.old_total_sol_value > 0,
        "old_total_sol_value should be captured post-sync"
    );
}

pub fn fixture_pool_and_lsl() -> (PoolState, Vec<u8>) {
    let pool_pk = Pubkey::new_from_array(POOL_STATE_ID);
    let pool_acc = ALL_FIXTURES
        .get(&pool_pk)
        .expect("missing pool state fixture");
    let pool = PoolStatePacked::of_acc_data(&pool_acc.data)
        .expect("pool packed")
        .into_pool_state();

    let lsl_pk = Pubkey::new_from_array(LST_STATE_LIST_ID);
    let lsl_acc = ALL_FIXTURES.get(&lsl_pk).expect("missing lsl fixture");

    (pool, lsl_acc.data.clone())
}

pub type StartRebalanceKeysBuilder =
    StartRebalanceIxAccs<[u8; 32], StartRebalanceIxPreKeysOwned, SvcCalcAccsAg, SvcCalcAccsAg>;

pub fn start_rebalance_ix_pre_keys_owned(
    rebalance_auth: [u8; 32],
    out_token_program: &[u8; 32],
    out_mint: [u8; 32],
    inp_mint: [u8; 32],
    withdraw_to: [u8; 32],
) -> StartRebalanceIxPreKeysOwned {
    let rebalance_record_pda = Pubkey::new_from_array(REBALANCE_RECORD_ID);

    NewStartRebalanceIxPreAccsBuilder::start()
        .with_rebalance_auth(rebalance_auth)
        .with_pool_state(POOL_STATE_ID)
        .with_lst_state_list(LST_STATE_LIST_ID)
        .with_rebalance_record(rebalance_record_pda.to_bytes())
        .with_out_lst_mint(out_mint)
        .with_inp_lst_mint(inp_mint)
        .with_out_pool_reserves(
            inf1_test_utils::find_pool_reserves_ata(out_token_program, &out_mint)
                .0
                .to_bytes(),
        )
        .with_inp_pool_reserves(
            inf1_test_utils::find_pool_reserves_ata(out_token_program, &inp_mint)
                .0
                .to_bytes(),
        )
        .with_withdraw_to(withdraw_to)
        .with_instructions(INSTRUCTIONS_SYSVAR_ID)
        .with_system_program(SYSTEM_PROGRAM_ID)
        .with_out_lst_token_program(*out_token_program)
        .build()
}

pub fn rebalance_ixs(
    builder: &StartRebalanceKeysBuilder,
    out_lst_index: u32,
    inp_lst_index: u32,
    amount: u64,
    min_starting_out_lst: u64,
    max_starting_inp_lst: u64,
) -> Vec<Instruction> {
    let start_args = StartRebalanceIxArgs {
        out_lst_index,
        inp_lst_index,
        amount,
        min_starting_out_lst,
        max_starting_inp_lst,
        accs: *builder,
    };

    let start_ix = Instruction {
        program_id: Pubkey::new_from_array(ID),
        accounts: keys_signer_writable_to_metas(
            builder.keys_owned().seq(),
            builder.is_signer().seq(),
            builder.is_writer().seq(),
        ),
        data: StartRebalanceIxData::new(start_args.to_full())
            .as_buf()
            .into(),
    };

    let end_accs = EndRebalanceIxAccs::from_start(*builder);
    let end_ix = Instruction {
        program_id: Pubkey::new_from_array(ID),
        accounts: keys_signer_writable_to_metas(
            end_accs.keys_owned().seq(),
            end_accs.is_signer().seq(),
            end_accs.is_writer().seq(),
        ),
        data: EndRebalanceIxData::new().as_buf().into(),
    };

    vec![start_ix, end_ix]
}

pub fn wsol_builder(
    rebalance_auth: [u8; 32],
    out_mint: [u8; 32],
    inp_mint: [u8; 32],
    withdraw_to: [u8; 32],
) -> StartRebalanceKeysBuilder {
    let ix_prefix = start_rebalance_ix_pre_keys_owned(
        rebalance_auth,
        &TOKENKEG_PROGRAM,
        out_mint,
        inp_mint,
        withdraw_to,
    );

    StartRebalanceKeysBuilder {
        ix_prefix,
        out_calc_prog: *SvcAgTy::Wsol(()).svc_program_id(),
        out_calc: SvcCalcAccsAg::Wsol(WsolCalcAccs),
        inp_calc_prog: *SvcAgTy::Wsol(()).svc_program_id(),
        inp_calc: SvcCalcAccsAg::Wsol(WsolCalcAccs),
    }
}

pub fn jupsol_wsol_builder(
    rebalance_auth: [u8; 32],
    out_mint: [u8; 32],
    inp_mint: [u8; 32],
    withdraw_to: [u8; 32],
) -> StartRebalanceKeysBuilder {
    let ix_prefix = start_rebalance_ix_pre_keys_owned(
        rebalance_auth,
        &TOKENKEG_PROGRAM,
        out_mint,
        inp_mint,
        withdraw_to,
    );

    StartRebalanceKeysBuilder {
        ix_prefix,
        out_calc_prog: *SvcAgTy::SanctumSplMulti(()).svc_program_id(),
        out_calc: jupsol_fixtures_svc_suf(),
        inp_calc_prog: *SvcAgTy::Wsol(()).svc_program_id(),
        inp_calc: SvcCalcAccsAg::Wsol(WsolCalcAccs),
    }
}

pub fn fixture_lst_state_data() -> (PoolState, LstStateListData, LstStateData, LstStateData) {
    let (pool, lst_state_bytes) = fixture_pool_and_lsl();

    let packed_list = LstStatePackedList::of_acc_data(&lst_state_bytes).expect("lst packed");
    let packed_states = &packed_list.0;

    let mut out_state = packed_states[JUPSOL_FIXTURE_LST_IDX].into_lst_state();
    out_state.sol_value_calculator = *SvcAgTy::Wsol(()).svc_program_id();

    let mut inp_state = packed_states
        .iter()
        .find(|s| s.into_lst_state().mint == WSOL_MINT.to_bytes())
        .expect("wsol fixture available")
        .into_lst_state();
    inp_state.sol_value_calculator = *SvcAgTy::Wsol(()).svc_program_id();

    let out_lsd = gen_lst_state(
        GenLstStateArgs {
            is_input_disabled: u8_to_bool(out_state.is_input_disabled),
            sol_value: out_state.sol_value,
            pks: NewLstStatePksBuilder::start()
                .with_mint(out_state.mint)
                .with_sol_value_calculator(out_state.sol_value_calculator)
                .build(),
            bumps: NewLstStateBumpsBuilder::start()
                .with_pool_reserves_bump(out_state.pool_reserves_bump)
                .with_protocol_fee_accumulator_bump(out_state.protocol_fee_accumulator_bump)
                .build(),
        },
        &TOKENKEG_PROGRAM,
    );

    let inp_lsd = gen_lst_state(
        GenLstStateArgs {
            is_input_disabled: u8_to_bool(inp_state.is_input_disabled),
            sol_value: inp_state.sol_value,
            pks: NewLstStatePksBuilder::start()
                .with_mint(inp_state.mint)
                .with_sol_value_calculator(inp_state.sol_value_calculator)
                .build(),
            bumps: NewLstStateBumpsBuilder::start()
                .with_pool_reserves_bump(inp_state.pool_reserves_bump)
                .with_protocol_fee_accumulator_bump(inp_state.protocol_fee_accumulator_bump)
                .build(),
        },
        &TOKENKEG_PROGRAM,
    );

    let mut lsl_data = LstStateListData {
        lst_state_list: lst_state_bytes,
        protocol_fee_accumulators: HashMap::new(),
        all_pool_reserves: HashMap::new(),
    };

    lsl_data.upsert(out_lsd);
    lsl_data.upsert(inp_lsd);

    (pool, lsl_data, out_lsd, inp_lsd)
}

#[allow(clippy::too_many_arguments)]
pub fn add_common_accounts(
    accounts: &mut Vec<PkAccountTup>,
    pool: &PoolState,
    lst_state_list: &[u8],
    pool_reserves_map: Option<&HashMap<[u8; 32], [u8; 32]>>,
    rebalance_auth: [u8; 32],
    out_mint: [u8; 32],
    inp_mint: [u8; 32],
    withdraw_to: [u8; 32],
    out_balance: u64,
    inp_balance: u64,
) {
    upsert_account(
        accounts,
        (
            LST_STATE_LIST_ID.into(),
            lst_state_list_account(lst_state_list.to_vec()),
        ),
    );
    upsert_account(accounts, (POOL_STATE_ID.into(), pool_state_account(*pool)));
    upsert_account(
        accounts,
        (
            Pubkey::new_from_array(rebalance_auth),
            Account {
                lamports: u64::MAX,
                owner: Pubkey::new_from_array(SYSTEM_PROGRAM_ID),
                ..Default::default()
            },
        ),
    );
    upsert_account(
        accounts,
        (
            Pubkey::new_from_array(out_mint),
            mock_mint(raw_mint(None, None, 0, 9)),
        ),
    );
    upsert_account(
        accounts,
        (
            Pubkey::new_from_array(inp_mint),
            mock_mint(raw_mint(None, None, 0, 9)),
        ),
    );
    upsert_account(
        accounts,
        (
            pool_reserves_map
                .and_then(|m| m.get(&out_mint).copied())
                .map(Pubkey::new_from_array)
                .unwrap_or_else(|| {
                    inf1_test_utils::find_pool_reserves_ata(&TOKENKEG_PROGRAM, &out_mint).0
                }),
            mock_token_acc(raw_token_acc(out_mint, POOL_STATE_ID, out_balance)),
        ),
    );
    upsert_account(
        accounts,
        (
            pool_reserves_map
                .and_then(|m| m.get(&inp_mint).copied())
                .map(Pubkey::new_from_array)
                .unwrap_or_else(|| {
                    inf1_test_utils::find_pool_reserves_ata(&TOKENKEG_PROGRAM, &inp_mint).0
                }),
            mock_token_acc(raw_token_acc(inp_mint, POOL_STATE_ID, inp_balance)),
        ),
    );
    upsert_account(
        accounts,
        (
            Pubkey::new_from_array(withdraw_to),
            mock_token_acc(raw_token_acc(out_mint, withdraw_to, 0)),
        ),
    );
}
