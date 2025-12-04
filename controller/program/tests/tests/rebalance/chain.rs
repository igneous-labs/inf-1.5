use std::{iter::once, ops::Neg};

use expect_test::expect;
use inf1_ctl_jiminy::{
    accounts::pool_state::PoolStateV2Packed,
    instructions::rebalance::{
        end::{EndRebalanceIxData, EndRebalanceIxPreKeysOwned},
        start::{
            NewStartRebalanceIxPreAccsBuilder, StartRebalanceIxData, StartRebalanceIxPreAccs,
            StartRebalanceIxPreKeysOwned, START_REBALANCE_IX_PRE_ACCS_IDX_INP_POOL_RESERVES,
            START_REBALANCE_IX_PRE_ACCS_IDX_LST_STATE_LIST,
            START_REBALANCE_IX_PRE_ACCS_IDX_OUT_POOL_RESERVES,
            START_REBALANCE_IX_PRE_ACCS_IDX_POOL_STATE,
            START_REBALANCE_IX_PRE_ACCS_IDX_REBALANCE_RECORD,
        },
    },
    keys::{INSTRUCTIONS_SYSVAR_ID, REBALANCE_RECORD_ID},
};
use inf1_std::{
    instructions::rebalance::{
        end::EndRebalanceIxAccs,
        start::{StartRebalanceIxAccs, StartRebalanceIxArgs},
    },
    quote::rebalance::{quote_rebalance_exact_out, RebalanceQuote, RebalanceQuoteArgs},
};
use inf1_svc_ag_core::{
    calc::SvcCalcAg,
    inf1_svc_lido_core::solido_legacy_core::TOKENKEG_PROGRAM,
    inf1_svc_wsol_core::{calc::WsolCalc, instructions::sol_val_calc::WsolCalcAccs},
    instructions::SvcCalcAccsAg,
    SvcAg, SvcAgTy,
};
use inf1_test_utils::{
    acc_bef_aft, assert_diffs_lst_state_list, assert_jiminy_prog_err, assert_token_acc_diffs,
    fill_mock_prog_accs, get_lst_state_list, get_token_account_amount, jupsol_fixture_svc_suf_accs,
    keys_signer_writable_to_metas, mock_instructions_sysvar, mock_sys_acc, mock_token_acc,
    mollusk_exec, pool_state_v2_account, raw_token_acc, token_acc_bal_diff_changed, AccountMap,
    KeyedUiAccount, LstStateListChanges, VerPoolState, JUPSOL_FIXTURE_LST_IDX,
    WSOL_FIXTURE_LST_IDX,
};
use jiminy_cpi::program_error::ProgramError;
use mollusk_svm::{program::keyed_account_for_system_program, Mollusk};
use sanctum_spl_token_jiminy::sanctum_spl_token_core::{
    instructions::transfer::{
        NewTransferIxAccsBuilder, TransferIxAccs, TransferIxData, TRANSFER_IX_IS_SIGNER,
        TRANSFER_IX_IS_WRITABLE,
    },
    state::account::RawTokenAccount,
};
use solana_account::Account;
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

use crate::common::{derive_svc_no_inf, header_lookahead, lst_state_lookahead, Cbs, SVM};

type StartAccs =
    StartRebalanceIxAccs<[u8; 32], StartRebalanceIxPreKeysOwned, SvcCalcAccsAg, SvcCalcAccsAg>;
type StartArgs =
    StartRebalanceIxArgs<[u8; 32], StartRebalanceIxPreKeysOwned, SvcCalcAccsAg, SvcCalcAccsAg>;
type EndAccs = EndRebalanceIxAccs<[u8; 32], EndRebalanceIxPreKeysOwned, SvcCalcAccsAg>;

fn fill_rebal_prog_accs(
    am: &mut AccountMap,
    StartAccs {
        inp_calc_prog,
        out_calc_prog,
        ..
    }: &StartAccs,
) {
    fill_mock_prog_accs(am, [*inp_calc_prog, *out_calc_prog]);
}

fn create_transfer_ix(transfer_accs: &TransferIxAccs<[u8; 32]>, amount: u64) -> Instruction {
    let transfer_ix_data = TransferIxData::new(amount);
    Instruction {
        program_id: Pubkey::new_from_array(TOKENKEG_PROGRAM),
        accounts: keys_signer_writable_to_metas(
            transfer_accs.0.iter(),
            TRANSFER_IX_IS_SIGNER.0.iter(),
            TRANSFER_IX_IS_WRITABLE.0.iter(),
        ),
        data: transfer_ix_data.as_buf().into(),
    }
}

fn jupsol_o_wsol_i_prefix_fixtures() -> StartRebalanceIxPreAccs<(Pubkey, Account)> {
    const MIGRATION_SLOT: u64 = 0;

    let accs = StartRebalanceIxPreAccs(
        NewStartRebalanceIxPreAccsBuilder::start()
            .with_pool_state("pool-state")
            .with_lst_state_list("lst-state-list")
            .with_out_lst_mint("jupsol-mint")
            .with_out_pool_reserves("jupsol-reserves")
            .with_inp_lst_mint("wsol-mint")
            .with_inp_pool_reserves("wsol-reserves")
            // filler
            .with_withdraw_to("wsol-mint")
            .with_instructions("wsol-mint")
            .with_out_lst_token_program("wsol-mint")
            .with_rebalance_auth("wsol-mint")
            .with_rebalance_record("wsol-mint")
            .with_system_program("wsol-mint")
            .build()
            .0
            .map(|n| KeyedUiAccount::from_test_fixtures_json(n).into_keyed_account()),
    );

    // Rebalance does not perform migration, but our fixtures are PoolStateV1,
    // so just patch it into v2 here
    let ps = accs.pool_state();
    let ps_addr = ps.0;
    let ps_acc =
        pool_state_v2_account(VerPoolState::from_acc_data(&ps.1.data).migrated(MIGRATION_SLOT));
    let accs = accs.with_pool_state((ps_addr, ps_acc));

    replace_fixture_fillers(accs)
}

/// Replace non-fixture filler accounts:
/// - instructions sysvar is empty and must be set after
/// - rebalance auth set to mock_sys_acc of pool_state fixture
/// - withdraw_to set to empty token acc owned by rebalance auth
fn replace_fixture_fillers(
    accs: StartRebalanceIxPreAccs<(Pubkey, Account)>,
) -> StartRebalanceIxPreAccs<(Pubkey, Account)> {
    const WITHDRAW_TO_FIXTURE: Pubkey =
        Pubkey::from_str_const("HKxhC3j5CfWRLiHkZutR42Q2SUctMjvY49w3n5wLViqC");
    const REBAL_AUTH_LAMPORTS: u64 = 1_000_000_000;

    let rebalance_auth_addr = PoolStateV2Packed::of_acc_data(&accs.pool_state().1.data)
        .unwrap()
        .into_pool_state_v2()
        .rebalance_authority;
    let withdraw_to_acc = mock_token_acc(raw_token_acc(
        accs.out_lst_mint().0.to_bytes(),
        rebalance_auth_addr,
        0,
    ));
    accs.with_withdraw_to((WITHDRAW_TO_FIXTURE, withdraw_to_acc))
        .with_instructions((INSTRUCTIONS_SYSVAR_ID.into(), Default::default()))
        .with_out_lst_token_program(mollusk_svm_programs_token::token::keyed_account())
        .with_rebalance_auth((
            rebalance_auth_addr.into(),
            mock_sys_acc(REBAL_AUTH_LAMPORTS),
        ))
        .with_rebalance_record((REBALANCE_RECORD_ID.into(), Default::default()))
        .with_system_program(keyed_account_for_system_program())
}

/// Currently assumes that StartRebalance is the first ix
/// and EndRebalance is the last ix
fn to_inp(
    start: &StartArgs,
    mid: impl IntoIterator<Item = Instruction>,
    end: &EndAccs,
    ams: impl IntoIterator<Item = AccountMap>,
) -> (Vec<Instruction>, AccountMap) {
    let start_ix = Instruction {
        program_id: Pubkey::new_from_array(inf1_ctl_jiminy::ID),
        accounts: keys_signer_writable_to_metas(
            start.accs.keys_owned().seq(),
            start.accs.is_signer().seq(),
            start.accs.is_writer().seq(),
        ),
        data: StartRebalanceIxData::new(start.to_full()).as_buf().into(),
    };

    let end_ix = Instruction {
        program_id: Pubkey::new_from_array(inf1_ctl_jiminy::ID),
        accounts: keys_signer_writable_to_metas(
            end.keys_owned().seq(),
            end.is_signer().seq(),
            end.is_writer().seq(),
        ),
        data: EndRebalanceIxData::as_buf().into(),
    };
    let ixs: Vec<_> = once(start_ix).chain(mid).chain(once(end_ix)).collect();

    let mut am = ams.into_iter().flat_map(|am| am.into_iter()).collect();
    fill_rebal_prog_accs(&mut am, &start.accs);
    am.insert(
        Pubkey::new_from_array(INSTRUCTIONS_SYSVAR_ID),
        // curr_ix=0, assumes StartRebalance is first ix
        mock_instructions_sysvar(&ixs, 0),
    );

    (ixs, am)
}

fn rebalance_test(
    svm: &Mollusk,
    bef: &AccountMap,
    ixs: &[Instruction],
    out_calc: &SvcCalcAg,
    inp_calc: &SvcCalcAg,
    expected_err: Option<impl Into<ProgramError>>,
) {
    let result = mollusk_exec(svm, ixs, bef);

    match expected_err {
        None => {
            let aft = result.unwrap().resulting_accounts;
            let clock = &svm.sysvars.clock;
            assert_correct_rebalance(bef, &aft, ixs, out_calc, inp_calc, clock.slot);
        }
        Some(e) => {
            assert_jiminy_prog_err(&result.unwrap_err(), e);
        }
    }
}

fn assert_correct_rebalance(
    bef: &AccountMap,
    aft: &AccountMap,
    ixs: &[Instruction],
    out_calc: &SvcCalcAg,
    inp_calc: &SvcCalcAg,
    slot: u64,
) {
    let start_ix = &ixs[0];

    let inf1_ctl_jiminy::instructions::rebalance::start::StartRebalanceIxArgs {
        out_lst_index,
        inp_lst_index,
        amount,
        ..
    } = StartRebalanceIxData::parse_no_discm(
        start_ix.data.split_first().unwrap().1.try_into().unwrap(),
    );
    let [out_lst_index, inp_lst_index] = [out_lst_index, inp_lst_index].map(|x| x as usize);

    // rebalance record should not exist in aft
    let rr_addr = start_ix.accounts[START_REBALANCE_IX_PRE_ACCS_IDX_REBALANCE_RECORD].pubkey;
    let rr_opt = aft.get(&rr_addr);
    assert!(
        rr_opt.map_or_else(|| true, |a| *a == Default::default()),
        "{rr_opt:?}"
    );

    // out reserves should go down by amount arg
    let [[out_reserves_bef, out_reserves_aft], [inp_reserves_bef, _inp_reserves_aft]] = [
        START_REBALANCE_IX_PRE_ACCS_IDX_OUT_POOL_RESERVES,
        START_REBALANCE_IX_PRE_ACCS_IDX_INP_POOL_RESERVES,
    ]
    .map(|i| {
        acc_bef_aft(&start_ix.accounts[i].pubkey, bef, aft)
            .map(|a| RawTokenAccount::of_acc_data(&a.data).unwrap())
    });
    assert_token_acc_diffs(
        out_reserves_bef,
        out_reserves_aft,
        &token_acc_bal_diff_changed(out_reserves_bef, i128::from(amount).neg()),
    );

    let [ps_addr, list_addr] = [
        START_REBALANCE_IX_PRE_ACCS_IDX_POOL_STATE,
        START_REBALANCE_IX_PRE_ACCS_IDX_LST_STATE_LIST,
    ]
    .map(|i| start_ix.accounts[i].pubkey);

    let [out_reserves_bef, inp_reserves_bef] =
        [out_reserves_bef, inp_reserves_bef].map(|a| u64::from_le_bytes(a.amount));

    let [mut list_bef, list_aft] =
        acc_bef_aft(&list_addr, bef, aft).map(|a| get_lst_state_list(&a.data));

    let cbs = [
        (out_lst_index, out_reserves_bef, out_calc),
        (inp_lst_index, inp_reserves_bef, inp_calc),
    ]
    .map(|(idx, balance, calc)| {
        let old_state = list_bef[idx];
        let ret = Cbs {
            calc,
            balance,
            old_sol_val: old_state.sol_value,
        };

        list_bef[idx] = lst_state_lookahead(old_state, balance, calc);

        ret
    });

    let [ps_bef, ps_aft] = acc_bef_aft(&ps_addr, bef, aft).map(|a| {
        PoolStateV2Packed::of_acc_data(&a.data)
            .unwrap()
            .into_pool_state_v2()
    });
    let ps_bef = header_lookahead(ps_bef, cbs, slot);

    assert!(
        ps_aft.total_sol_value >= ps_bef.total_sol_value,
        "{} < {}",
        ps_aft.total_sol_value,
        ps_bef.total_sol_value
    );
    let tsv_inc = ps_aft.total_sol_value - ps_bef.total_sol_value;

    let (list_diffs, inp_svc) = LstStateListChanges::new(&list_bef)
        .with_det_svc_by_mint(&list_aft[inp_lst_index].mint, &list_aft);
    let (list_diffs, out_svc) =
        list_diffs.with_det_svc_by_mint(&list_aft[out_lst_index].mint, &list_aft);

    // assert everything else other than sol value didnt change
    assert_diffs_lst_state_list(list_diffs.build(), list_bef, list_aft);

    assert!(inp_svc >= 0);
    assert!(out_svc <= 0);

    assert_eq!(
        inp_svc + out_svc,
        i128::from(tsv_inc),
        "{} - {} != {}",
        inp_svc,
        out_svc.neg(),
        tsv_inc
    );
}

#[test]
fn rebal_jupsol_o_wsol_i_fixture_basic() {
    const DONOR_TOKEN_ACC_ADDR: Pubkey =
        Pubkey::from_str_const("9hGZcUjDQ752puJN25Bvmerj6Rt1bjoU31g3D5g8Eztt");
    const AMOUNT: u64 = 100_000;
    const CURR_EPOCH: u64 = 0;

    let prefix_am = jupsol_o_wsol_i_prefix_fixtures();
    let ix_prefix =
        StartRebalanceIxPreAccs(prefix_am.0.each_ref().map(|(addr, _)| addr.to_bytes()));
    let (out_accs, out_am) = jupsol_fixture_svc_suf_accs();
    let out_accs = SvcAg::SanctumSplMulti(out_accs);
    let inp_accs = SvcAg::Wsol(WsolCalcAccs);

    let start_accs = StartAccs {
        ix_prefix,
        out_calc_prog: *SvcAgTy::SanctumSplMulti(()).svc_program_id(),
        out_calc: out_accs,
        inp_calc_prog: *SvcAgTy::Wsol(()).svc_program_id(),
        inp_calc: inp_accs,
    };
    let start_args = StartArgs {
        out_lst_index: JUPSOL_FIXTURE_LST_IDX.try_into().unwrap(),
        inp_lst_index: WSOL_FIXTURE_LST_IDX.try_into().unwrap(),
        amount: AMOUNT,
        min_starting_out_lst: 0,
        max_starting_inp_lst: u64::MAX,
        accs: start_accs,
    };

    let prefix_am: AccountMap = prefix_am.0.into_iter().collect();
    let out_calc = derive_svc_no_inf(&out_am, &out_accs, CURR_EPOCH);
    let inp_calc = SvcAg::Wsol(WsolCalc);
    let [inp_reserves, out_reserves] =
        [ix_prefix.inp_pool_reserves(), ix_prefix.out_pool_reserves()]
            .map(|a| get_token_account_amount(&prefix_am[&(*a).into()].data));

    let RebalanceQuote { inp, out, .. } = quote_rebalance_exact_out(RebalanceQuoteArgs {
        amt: AMOUNT,
        inp_reserves,
        out_reserves,
        inp_mint: *ix_prefix.inp_lst_mint(),
        out_mint: *ix_prefix.out_lst_mint(),
        inp_calc,
        out_calc,
    })
    .unwrap();

    let (ixs, bef) = to_inp(
        &start_args,
        [create_transfer_ix(
            &NewTransferIxAccsBuilder::start()
                .with_auth(*ix_prefix.rebalance_auth())
                .with_dst(*ix_prefix.inp_pool_reserves())
                .with_src(DONOR_TOKEN_ACC_ADDR.to_bytes())
                .build(),
            inp,
        )],
        &EndAccs::from_start(start_accs),
        [
            prefix_am,
            out_am,
            once((
                DONOR_TOKEN_ACC_ADDR,
                mock_token_acc(raw_token_acc(
                    *ix_prefix.inp_lst_mint(),
                    *ix_prefix.rebalance_auth(),
                    inp,
                )),
            ))
            .collect(),
        ],
    );

    SVM.with(|svm| rebalance_test(svm, &bef, &ixs, &out_calc, &inp_calc, None::<ProgramError>));

    expect![[r#"
        (
            111331,
            100000,
        )
    "#]]
    .assert_debug_eq(&(inp, out));
}

// struct TestFixture {
//     pool: PoolState,
//     lsl: LstStateListData,
//     out_lsd: LstStateData,
//     inp_lsd: LstStateData,
//     builder: StartRebalanceKeysBuilder,
//     withdraw_to: [u8; 32],
//     out_idx: u32,
//     inp_idx: u32,
// }

// fn setup_test_fixture() -> TestFixture {
//     let (pool, mut lsl, mut out_lsd, mut inp_lsd) = fixture_lst_state_data();
//     let withdraw_to = Pubkey::new_unique().to_bytes();
//     let builder = jupsol_wsol_builder(
//         pool.rebalance_authority,
//         out_lsd.lst_state.mint,
//         inp_lsd.lst_state.mint,
//         withdraw_to,
//     );
//     out_lsd.lst_state.sol_value_calculator = builder.out_calc_prog;
//     inp_lsd.lst_state.sol_value_calculator = builder.inp_calc_prog;
//     let out_idx = lsl.upsert(out_lsd) as u32;
//     let inp_idx = lsl.upsert(inp_lsd) as u32;

//     TestFixture {
//         pool,
//         lsl,
//         out_lsd,
//         inp_lsd,
//         builder,
//         withdraw_to,
//         out_idx,
//         inp_idx,
//     }
// }

// struct OwnerAccounts {
//     owner: [u8; 32],
//     owner_token_account: [u8; 32],
//     owner_balance: u64,
// }

// fn setup_owner_accounts(balance: u64) -> OwnerAccounts {
//     OwnerAccounts {
//         owner: Pubkey::new_unique().to_bytes(),
//         owner_token_account: Pubkey::new_unique().to_bytes(),
//         owner_balance: balance,
//     }
// }

// fn standard_reserves(amount: u64) -> (u64, u64) {
//     (amount * 2, amount * 2)
// }

// fn setup_basic_rebalance_test(
//     fixture: &TestFixture,
//     amount: u64,
//     min_starting_out_lst: u64,
//     max_starting_inp_lst: u64,
// ) -> (Vec<Instruction>, AccountMap) {
//     let (out_reserves, inp_reserves) = standard_reserves(amount);
//     let instructions = rebalance_ixs(
//         &fixture.builder,
//         fixture.out_idx,
//         fixture.inp_idx,
//         amount,
//         min_starting_out_lst,
//         max_starting_inp_lst,
//     );
//     let owner_accs = setup_owner_accounts(0);
//     let accounts = setup_rebalance_transaction_accounts(
//         fixture,
//         &instructions,
//         out_reserves,
//         inp_reserves,
//         &owner_accs,
//     );
//     (instructions, accounts)
// }

// /// Calculates the input token amount for a JupSOL -> WSOL rebalance
// fn calculate_jupsol_wsol_inp_amount(
//     out_lst_amount: u64,
//     out_reserves: u64,
//     inp_reserves: u64,
//     out_mint: [u8; 32],
//     inp_mint: [u8; 32],
// ) -> u64 {
//     let (_, jupsol_pool_acc) =
//         KeyedUiAccount::from_test_fixtures_json("jupsol-pool.json").into_keyed_account();
//     let jupsol_stakepool = StakePool::borsh_de(jupsol_pool_acc.data.as_slice()).unwrap();

//     let inp_calc = WsolCalc;
//     let out_calc = SplCalc::new(&jupsol_stakepool, 0);

//     let quote = quote_rebalance_exact_out(RebalanceQuoteArgs {
//         amt: out_lst_amount,
//         inp_reserves,
//         out_reserves,
//         inp_mint,
//         out_mint,
//         inp_calc,
//         out_calc,
//     })
//     .expect("quote should succeed");

//     quote.inp
// }

// /// Creates the full account set required for StartRebalance → Token Transfer → EndRebalance transaction
// fn setup_rebalance_transaction_accounts(
//     fixture: &TestFixture,
//     instructions: &[Instruction],
//     out_balance: u64,
//     inp_balance: u64,
//     owner_accs: &OwnerAccounts,
// ) -> AccountMap {
//     let mut accounts: AccountMap =
//         fixtures_accounts_opt_cloned(fixture.builder.keys_owned().seq().copied());

//     add_common_accounts(
//         &mut accounts,
//         &fixture.pool,
//         &fixture.lsl.lst_state_list,
//         Some(&fixture.lsl.all_pool_reserves),
//         fixture.pool.rebalance_authority,
//         fixture.out_lsd.lst_state.mint,
//         fixture.inp_lsd.lst_state.mint,
//         fixture.withdraw_to,
//         out_balance,
//         inp_balance,
//     );

//     let (sys_prog_pk, sys_prog_acc) = keyed_account_for_system_program();
//     accounts.insert(sys_prog_pk, sys_prog_acc);

//     accounts.insert(
//         Pubkey::new_from_array(owner_accs.owner),
//         mock_sys_acc(100_000_000_000),
//     );

//     accounts.insert(
//         Pubkey::new_from_array(owner_accs.owner_token_account),
//         mock_token_acc(raw_token_acc(
//             fixture.inp_lsd.lst_state.mint,
//             owner_accs.owner,
//             owner_accs.owner_balance,
//         )),
//     );

//     accounts.insert(
//         Pubkey::new_from_array(INSTRUCTIONS_SYSVAR_ID),
//         mock_instructions_sysvar(instructions, 0),
//     );

//     accounts.insert(
//         Pubkey::new_from_array(REBALANCE_RECORD_ID),
//         Account::default(),
//     );

//     accounts
// }

// /// Creates an instruction chain with StartRebalance → Token Transfer → EndRebalance instructions.
// fn build_rebalance_instruction_chain(
//     fixture: &TestFixture,
//     owner_accs: &OwnerAccounts,
//     out_lst_amount: u64,
//     owner_transfer_amount: u64,
// ) -> Vec<Instruction> {
//     let mut instructions = rebalance_ixs(
//         &fixture.builder,
//         fixture.out_idx,
//         fixture.inp_idx,
//         out_lst_amount,
//         0,
//         u64::MAX,
//     );

//     let transfer_ix = create_transfer_ix(
//         owner_accs.owner,
//         owner_accs.owner_token_account,
//         fixture.inp_lsd.pool_reserves,
//         owner_transfer_amount,
//     );

//     // Put transfer ix between start and end
//     instructions.insert(1, transfer_ix);

//     instructions
// }

// fn execute_rebalance_transaction(
//     amount: u64,
//     out_reserves: Option<u64>,
//     inp_reserves: Option<u64>,
// ) -> (AccountMap, InstructionResult, u64) {
//     silence_mollusk_logs();

//     let fixture = setup_test_fixture();

//     let out_reserves = out_reserves.unwrap_or(amount * 2);
//     let inp_reserves = inp_reserves.unwrap_or(amount * 2);

//     let owner_transfer_amount = calculate_jupsol_wsol_inp_amount(
//         amount,
//         out_reserves,
//         inp_reserves,
//         fixture.out_lsd.lst_state.mint,
//         fixture.inp_lsd.lst_state.mint,
//     );

//     let owner_accs = setup_owner_accounts(owner_transfer_amount);

//     let instructions =
//         build_rebalance_instruction_chain(&fixture, &owner_accs, amount, owner_transfer_amount);

//     let accounts = setup_rebalance_transaction_accounts(
//         &fixture,
//         &instructions,
//         out_reserves,
//         inp_reserves,
//         &owner_accs,
//     );

//     let accs_bef = accounts.clone();

//     let mut accs_vec: Vec<_> = accounts.iter().map(|(k, v)| (*k, v.clone())).collect();
//     accs_vec.sort_by_key(|(k, _)| *k);
//     let result = SVM.with(|svm| svm.process_instruction_chain(&instructions, &accs_vec));

//     // Run StartRebalance ix separately to extract old_total_sol_value
//     // from RebalanceRecord
//     let (_, start_result) = SVM.with(|svm| mollusk_exec(svm, &instructions[0], &accs_bef));
//     let rr_aft = start_result
//         .resulting_accounts
//         .iter()
//         .find(|(pk, _)| pk.to_bytes() == REBALANCE_RECORD_ID)
//         .map(|(_, acc)| acc)
//         .expect("rebalance record after start");
//     let rebalance_record =
//         unsafe { RebalanceRecord::of_acc_data(&rr_aft.data) }.expect("rebalance record");

//     (accs_bef, result, rebalance_record.old_total_sol_value)
// }

// /// Validate that the transaction succeeded,
// /// the pool state is not rebalancing before or after,
// /// the pool did not lose SOL value,
// /// the RebalanceRecord is properly closed,
// /// and lamports are balanced.
// fn assert_rebalance_transaction_success(
//     accs_bef: &AccountMap,
//     result: &InstructionResult,
//     old_total_sol_value: u64,
// ) {
//     assert_eq!(result.program_result, ProgramResult::Success);

//     let aft: AccountMap = result.resulting_accounts.clone().into_iter().collect();
//     let [pool_state_bef, pool_state_aft] =
//         acc_bef_aft(&Pubkey::new_from_array(POOL_STATE_ID), accs_bef, &aft).map(|a| {
//             PoolStatePacked::of_acc_data(&a.data)
//                 .unwrap()
//                 .into_pool_state()
//         });

//     assert_eq!(pool_state_bef.is_rebalancing, 0);
//     assert_eq!(pool_state_aft.is_rebalancing, 0);
//     assert!(pool_state_aft.total_sol_value >= old_total_sol_value);

//     let rr_aft = aft
//         .iter()
//         .find(|(pk, _)| pk.to_bytes() == REBALANCE_RECORD_ID);
//     assert_eq!(rr_aft.unwrap().1.lamports, 0);

//     assert_balanced(accs_bef, &aft);
// }

// #[test]
// fn missing_end_rebalance() {
//     silence_mollusk_logs();

//     let fixture = setup_test_fixture();
//     let (out_reserves, inp_reserves) = standard_reserves(100_000);

//     let mut instructions = rebalance_ixs(
//         &fixture.builder,
//         fixture.out_idx,
//         fixture.inp_idx,
//         100_000,
//         0,
//         u64::MAX,
//     );
//     // Remove EndRebalance
//     instructions.pop();

//     let owner_accs = setup_owner_accounts(0);

//     let accounts = setup_rebalance_transaction_accounts(
//         &fixture,
//         &instructions,
//         out_reserves,
//         inp_reserves,
//         &owner_accs,
//     );

//     let (_, result) = SVM.with(|svm| mollusk_exec(svm, &instructions[0], &accounts));

//     assert_jiminy_prog_err(
//         &result.program_result,
//         Inf1CtlCustomProgErr(NoSucceedingEndRebalance),
//     );
// }

// #[test]
// fn wrong_end_mint() {
//     silence_mollusk_logs();

//     let fixture = setup_test_fixture();
//     let (out_reserves, inp_reserves) = standard_reserves(100_000);

//     let mut instructions = rebalance_ixs(
//         &fixture.builder,
//         fixture.out_idx,
//         fixture.inp_idx,
//         100_000,
//         0,
//         u64::MAX,
//     );

//     // Change EndRebalance instruction to use wrong inp_lst_mint
//     if let Some(end_ix) = instructions.get_mut(1) {
//         if end_ix.accounts.len() > END_REBALANCE_IX_PRE_ACCS_IDX_INP_LST_MINT {
//             end_ix.accounts[END_REBALANCE_IX_PRE_ACCS_IDX_INP_LST_MINT].pubkey =
//                 Pubkey::new_unique();
//         }
//     }

//     let owner_accs = setup_owner_accounts(0);

//     let accounts = setup_rebalance_transaction_accounts(
//         &fixture,
//         &instructions,
//         out_reserves,
//         inp_reserves,
//         &owner_accs,
//     );

//     let (_, result) = SVM.with(|svm| mollusk_exec(svm, &instructions[0], &accounts));

//     assert_jiminy_prog_err(
//         &result.program_result,
//         Inf1CtlCustomProgErr(NoSucceedingEndRebalance),
//     );
// }

// #[test]
// fn no_transfer() {
//     silence_mollusk_logs();

//     let fixture = setup_test_fixture();
//     let (instructions, accounts) = setup_basic_rebalance_test(&fixture, 100_000, 0, u64::MAX);

//     let mut accs_vec: Vec<_> = accounts.iter().map(|(k, v)| (*k, v.clone())).collect();
//     accs_vec.sort_by_key(|(k, _)| *k);
//     let result = SVM.with(|svm| svm.process_instruction_chain(&instructions, &accs_vec));

//     assert_jiminy_prog_err(
//         &result.program_result,
//         Inf1CtlCustomProgErr(PoolWouldLoseSolValue),
//     );
// }

// #[test]
// fn insufficient_transfer() {
//     silence_mollusk_logs();

//     let amount = 100_000;
//     let fixture = setup_test_fixture();
//     let (out_reserves, inp_reserves) = standard_reserves(amount);

//     let required_amount = calculate_jupsol_wsol_inp_amount(
//         amount,
//         out_reserves,
//         inp_reserves,
//         fixture.out_lsd.lst_state.mint,
//         fixture.inp_lsd.lst_state.mint,
//     );

//     let insufficient_amount = required_amount / 2;
//     let owner_accs = setup_owner_accounts(insufficient_amount);

//     let instructions =
//         build_rebalance_instruction_chain(&fixture, &owner_accs, amount, insufficient_amount);

//     let accounts = setup_rebalance_transaction_accounts(
//         &fixture,
//         &instructions,
//         out_reserves,
//         inp_reserves,
//         &owner_accs,
//     );

//     let mut accs_vec: Vec<_> = accounts.iter().map(|(k, v)| (*k, v.clone())).collect();
//     accs_vec.sort_by_key(|(k, _)| *k);
//     let result = SVM.with(|svm| svm.process_instruction_chain(&instructions, &accs_vec));

//     assert_jiminy_prog_err(
//         &result.program_result,
//         Inf1CtlCustomProgErr(PoolWouldLoseSolValue),
//     );
// }

// #[test]
// fn slippage_min_out_violated() {
//     silence_mollusk_logs();

//     let fixture = setup_test_fixture();

//     let (instructions, accounts) =
//         setup_basic_rebalance_test(&fixture, 100_000, u64::MAX, u64::MAX);

//     let (_, result) = SVM.with(|svm| mollusk_exec(svm, &instructions[0], &accounts));

//     assert_jiminy_prog_err(
//         &result.program_result,
//         Inf1CtlCustomProgErr(SlippageToleranceExceeded),
//     );
// }

// #[test]
// fn slippage_max_inp_violated() {
//     silence_mollusk_logs();

//     let fixture = setup_test_fixture();

//     let (instructions, accounts) = setup_basic_rebalance_test(&fixture, 100_000, 0, 1);

//     let (_, result) = SVM.with(|svm| mollusk_exec(svm, &instructions[0], &accounts));

//     assert_jiminy_prog_err(
//         &result.program_result,
//         Inf1CtlCustomProgErr(SlippageToleranceExceeded),
//     );
// }

// #[test]
// fn multi_instruction_transfer_chain() {
//     silence_mollusk_logs();

//     let amount = 100_000;
//     let fixture = setup_test_fixture();
//     let (out_reserves, inp_reserves) = standard_reserves(amount);

//     let total_transfer = calculate_jupsol_wsol_inp_amount(
//         amount,
//         out_reserves,
//         inp_reserves,
//         fixture.out_lsd.lst_state.mint,
//         fixture.inp_lsd.lst_state.mint,
//     );

//     let owner_accs = setup_owner_accounts(total_transfer);

//     let mut instructions = rebalance_ixs(
//         &fixture.builder,
//         fixture.out_idx,
//         fixture.inp_idx,
//         amount,
//         0,
//         u64::MAX,
//     );

//     let transfer1 = total_transfer / 3;
//     let transfer2 = total_transfer / 3;
//     let transfer3 = total_transfer - transfer1 - transfer2;

//     let transfer_ix1 = create_transfer_ix(
//         owner_accs.owner,
//         owner_accs.owner_token_account,
//         fixture.inp_lsd.pool_reserves,
//         transfer1,
//     );
//     let transfer_ix2 = create_transfer_ix(
//         owner_accs.owner,
//         owner_accs.owner_token_account,
//         fixture.inp_lsd.pool_reserves,
//         transfer2,
//     );
//     let transfer_ix3 = create_transfer_ix(
//         owner_accs.owner,
//         owner_accs.owner_token_account,
//         fixture.inp_lsd.pool_reserves,
//         transfer3,
//     );

//     instructions.insert(1, transfer_ix1);
//     instructions.insert(2, transfer_ix2);
//     instructions.insert(3, transfer_ix3);

//     let accounts = setup_rebalance_transaction_accounts(
//         &fixture,
//         &instructions,
//         out_reserves,
//         inp_reserves,
//         &owner_accs,
//     );

//     let accs_bef = accounts.clone();

//     let mut accs_vec: Vec<_> = accounts.iter().map(|(k, v)| (*k, v.clone())).collect();
//     accs_vec.sort_by_key(|(k, _)| *k);
//     let result = SVM.with(|svm| svm.process_instruction_chain(&instructions, &accs_vec));

//     let (_, start_result) = SVM.with(|svm| mollusk_exec(svm, &instructions[0], &accs_bef));
//     let rr_aft = start_result
//         .resulting_accounts
//         .iter()
//         .find(|(pk, _)| pk.to_bytes() == REBALANCE_RECORD_ID)
//         .map(|(_, acc)| acc)
//         .expect("rebalance record after start");
//     let rebalance_record =
//         unsafe { RebalanceRecord::of_acc_data(&rr_aft.data) }.expect("rebalance record");

//     assert_rebalance_transaction_success(&accs_bef, &result, rebalance_record.old_total_sol_value);
// }

// #[test]
// fn rebalance_chain_updates_reserves_correctly() {
//     silence_mollusk_logs();

//     let amount = 100_000;
//     let fixture = setup_test_fixture();
//     let (out_reserves, inp_reserves) = standard_reserves(amount);

//     let transfer_amount = calculate_jupsol_wsol_inp_amount(
//         amount,
//         out_reserves,
//         inp_reserves,
//         fixture.out_lsd.lst_state.mint,
//         fixture.inp_lsd.lst_state.mint,
//     );

//     let owner_accs = setup_owner_accounts(transfer_amount);

//     let instructions =
//         build_rebalance_instruction_chain(&fixture, &owner_accs, amount, transfer_amount);

//     let accounts = setup_rebalance_transaction_accounts(
//         &fixture,
//         &instructions,
//         out_reserves,
//         inp_reserves,
//         &owner_accs,
//     );

//     let accs_bef = accounts.clone();

//     let mut accs_vec: Vec<_> = accounts.iter().map(|(k, v)| (*k, v.clone())).collect();
//     accs_vec.sort_by_key(|(k, _)| *k);
//     let result = SVM.with(|svm| svm.process_instruction_chain(&instructions, &accs_vec));

//     assert_eq!(result.program_result, ProgramResult::Success);

//     let aft: AccountMap = result.resulting_accounts.into_iter().collect();
//     let [out_reserves_bef, out_reserves_aft] = acc_bef_aft(
//         &Pubkey::new_from_array(fixture.out_lsd.pool_reserves),
//         &accs_bef,
//         &aft,
//     )
//     .map(|a| get_token_account_amount(&a.data));

//     let [inp_reserves_bef, inp_reserves_aft] = acc_bef_aft(
//         &Pubkey::new_from_array(fixture.inp_lsd.pool_reserves),
//         &accs_bef,
//         &aft,
//     )
//     .map(|a| get_token_account_amount(&a.data));

//     let [withdraw_to_bef, withdraw_to_aft] = acc_bef_aft(
//         &Pubkey::new_from_array(fixture.withdraw_to),
//         &accs_bef,
//         &aft,
//     )
//     .map(|a| get_token_account_amount(&a.data));

//     assert_eq!(
//         out_reserves_aft,
//         out_reserves_bef - amount,
//         "out reserves should decrease by withdrawal amount"
//     );
//     assert_eq!(
//         inp_reserves_aft,
//         inp_reserves_bef + transfer_amount,
//         "inp reserves should increase by transfer amount"
//     );
//     assert_eq!(
//         withdraw_to_aft,
//         withdraw_to_bef + amount,
//         "withdraw_to should receive withdrawn LST"
//     );

//     assert_balanced(&accs_bef, &aft);
// }

// #[test]
// fn rebalance_record_lifecycle() {
//     silence_mollusk_logs();

//     let amount = 100_000;

//     let (accs_bef, result, old_total_sol_value) = execute_rebalance_transaction(amount, None, None);

//     assert_eq!(result.program_result, ProgramResult::Success);

//     let aft: AccountMap = result.resulting_accounts.clone().into_iter().collect();
//     let [pool_state_bef, pool_state_aft] =
//         acc_bef_aft(&Pubkey::new_from_array(POOL_STATE_ID), &accs_bef, &aft).map(|a| {
//             PoolStatePacked::of_acc_data(&a.data)
//                 .expect("pool state")
//                 .into_pool_state()
//         });

//     assert_eq!(pool_state_bef.is_rebalancing, 0);

//     let rr_bef = accs_bef
//         .iter()
//         .find(|(pk, _)| pk.to_bytes() == REBALANCE_RECORD_ID);
//     assert_eq!(
//         rr_bef.map(|(_, acc)| acc.lamports).unwrap(),
//         0,
//         "rebalance record should not exist initially"
//     );

//     assert_eq!(pool_state_bef.is_rebalancing, 0);
//     assert_eq!(pool_state_aft.is_rebalancing, 0);

//     assert!(pool_state_aft.total_sol_value >= old_total_sol_value);

//     let rr_aft = aft
//         .iter()
//         .find(|(pk, _)| pk.to_bytes() == REBALANCE_RECORD_ID);
//     assert_eq!(rr_aft.unwrap().1.lamports, 0);

//     // Verify RebalanceRecord creation by executing just StartRebalance
//     let fixture2 = setup_test_fixture();
//     let (start_ixs, start_accounts) = setup_basic_rebalance_test(&fixture2, amount, 0, u64::MAX);

//     let (_, start_result) = SVM.with(|svm| mollusk_exec(svm, &start_ixs[0], &start_accounts));
//     assert_eq!(start_result.program_result, ProgramResult::Success);

//     let start_aft: AccountMap = start_result.resulting_accounts.into_iter().collect();
//     let [pool_state_bef, pool_state_aft] = acc_bef_aft(
//         &Pubkey::new_from_array(POOL_STATE_ID),
//         &start_accounts,
//         &start_aft,
//     )
//     .map(|a| {
//         PoolStatePacked::of_acc_data(&a.data)
//             .expect("pool state")
//             .into_pool_state()
//     });

//     assert_eq!(pool_state_bef.is_rebalancing, 0);
//     assert_eq!(pool_state_aft.is_rebalancing, 1);

//     let rr_aft = start_aft
//         .iter()
//         .find(|(pk, _)| pk.to_bytes() == REBALANCE_RECORD_ID)
//         .map(|(_, acc)| acc)
//         .expect("rebalance record after start");

//     assert!(rr_aft.lamports > 0);

//     let rebalance_record =
//         unsafe { RebalanceRecord::of_acc_data(&rr_aft.data) }.expect("rebalance record");

//     assert_eq!(rebalance_record.inp_lst_index, fixture2.inp_idx);

//     assert!(rebalance_record.old_total_sol_value > 0);

//     assert_balanced(&accs_bef, &aft);
// }

// #[test]
// fn pool_already_rebalancing() {
//     silence_mollusk_logs();

//     let fixture = setup_test_fixture();
//     let owner_accs = setup_owner_accounts(0);
//     let (out_reserves, inp_reserves) = standard_reserves(100_000);

//     let first_instructions = rebalance_ixs(
//         &fixture.builder,
//         fixture.out_idx,
//         fixture.inp_idx,
//         100_000,
//         0,
//         u64::MAX,
//     );

//     let accounts = setup_rebalance_transaction_accounts(
//         &fixture,
//         &first_instructions,
//         out_reserves,
//         inp_reserves,
//         &owner_accs,
//     );

//     // Execute first StartRebalance instruction to set pool.is_rebalancing = 1
//     let (_, result) = SVM.with(|svm| mollusk_exec(svm, &first_instructions[0], &accounts));
//     assert_eq!(result.program_result, ProgramResult::Success);

//     let pool_state_aft = result
//         .resulting_accounts
//         .iter()
//         .find(|(pk, _)| pk.to_bytes() == POOL_STATE_ID)
//         .map(|(_, acc)| {
//             PoolStatePacked::of_acc_data(&acc.data)
//                 .expect("pool state")
//                 .into_pool_state()
//         })
//         .expect("pool state");
//     assert_eq!(pool_state_aft.is_rebalancing, 1);

//     let second_instructions = rebalance_ixs(
//         &fixture.builder,
//         fixture.out_idx,
//         fixture.inp_idx,
//         100_000,
//         0,
//         u64::MAX,
//     );

//     let aft: AccountMap = result.resulting_accounts.into_iter().collect();
//     let mut accounts_with_second_ix: AccountMap = aft.clone();
//     accounts_with_second_ix.insert(
//         Pubkey::new_from_array(INSTRUCTIONS_SYSVAR_ID),
//         mock_instructions_sysvar(&second_instructions, 0),
//     );

//     // Execute another StartRebalance instruction
//     let (_, result2) =
//         SVM.with(|svm| mollusk_exec(svm, &second_instructions[0], &accounts_with_second_ix));

//     assert_jiminy_prog_err(
//         &result2.program_result,
//         Inf1CtlCustomProgErr(PoolRebalancing),
//     );
// }

// #[test]
// fn unauthorized_rebalance_authority() {
//     silence_mollusk_logs();

//     let fixture = setup_test_fixture();
//     let owner_accs = setup_owner_accounts(0);
//     let (out_reserves, inp_reserves) = standard_reserves(100_000);

//     let unauthorized_pk = Pubkey::new_unique().to_bytes();
//     let unauthorized_builder = jupsol_wsol_builder(
//         unauthorized_pk,
//         fixture.out_lsd.lst_state.mint,
//         fixture.inp_lsd.lst_state.mint,
//         fixture.withdraw_to,
//     );

//     let instructions = rebalance_ixs(
//         &unauthorized_builder,
//         fixture.out_idx,
//         fixture.inp_idx,
//         100_000,
//         0,
//         u64::MAX,
//     );

//     let mut accounts = setup_rebalance_transaction_accounts(
//         &fixture,
//         &instructions,
//         out_reserves,
//         inp_reserves,
//         &owner_accs,
//     );

//     accounts.insert(
//         Pubkey::new_from_array(unauthorized_pk),
//         mock_sys_acc(100_000_000_000),
//     );

//     let mut accs_vec: Vec<_> = accounts.iter().map(|(k, v)| (*k, v.clone())).collect();
//     accs_vec.sort_by_key(|(k, _)| *k);
//     let result = SVM.with(|svm| svm.process_instruction_chain(&instructions, &accs_vec));

//     assert_jiminy_prog_err(&result.program_result, INVALID_ARGUMENT);
// }

// proptest! {
//   #[test]
//   fn rebalance_transaction_various_amounts_any(
//       amount in 1u64..=1_000_000_000,
//       out_reserve_multiplier in 2u64..=100,
//       inp_reserve_multiplier in 2u64..=100,
//   ) {
//       let out_reserves = amount.saturating_mul(out_reserve_multiplier);
//       let inp_reserves = amount.saturating_mul(inp_reserve_multiplier);

//       let (accs_bef, result, old_total_sol_value) =
//           execute_rebalance_transaction(amount, Some(out_reserves), Some(inp_reserves));

//       assert_rebalance_transaction_success(&accs_bef, &result, old_total_sol_value);
//   }
// }
