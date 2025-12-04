use std::{iter::once, ops::Neg};

use expect_test::expect;
use inf1_ctl_jiminy::{
    accounts::pool_state::PoolStateV2Packed,
    err::Inf1CtlErr,
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
    program_err::Inf1CtlCustomProgErr,
    typedefs::u8bool::U8Bool,
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
use jiminy_cpi::program_error::{ProgramError, INVALID_ARGUMENT};
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

const DONOR_TOKEN_ACC_ADDR: Pubkey =
    Pubkey::from_str_const("9hGZcUjDQ752puJN25Bvmerj6Rt1bjoU31g3D5g8Eztt");

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

fn jupsol_o_wsol_i_fixture_accs() -> (StartAccs, AccountMap) {
    let prefix_am = jupsol_o_wsol_i_prefix_fixtures();
    let ix_prefix =
        StartRebalanceIxPreAccs(prefix_am.0.each_ref().map(|(addr, _)| addr.to_bytes()));
    let (out_accs, mut out_am) = jupsol_fixture_svc_suf_accs();
    let out_accs = SvcAg::SanctumSplMulti(out_accs);
    let inp_accs = SvcAg::Wsol(WsolCalcAccs);

    out_am.extend(prefix_am.0);

    (
        StartAccs {
            ix_prefix,
            out_calc_prog: *SvcAgTy::SanctumSplMulti(()).svc_program_id(),
            out_calc: out_accs,
            inp_calc_prog: *SvcAgTy::Wsol(()).svc_program_id(),
            inp_calc: inp_accs,
        },
        out_am,
    )
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

fn to_start_ix(start: &StartArgs) -> Instruction {
    Instruction {
        program_id: Pubkey::new_from_array(inf1_ctl_jiminy::ID),
        accounts: keys_signer_writable_to_metas(
            start.accs.keys_owned().seq(),
            start.accs.is_signer().seq(),
            start.accs.is_writer().seq(),
        ),
        data: StartRebalanceIxData::new(start.to_full()).as_buf().into(),
    }
}

fn to_end_ix(end: &EndAccs) -> Instruction {
    Instruction {
        program_id: Pubkey::new_from_array(inf1_ctl_jiminy::ID),
        accounts: keys_signer_writable_to_metas(
            end.keys_owned().seq(),
            end.is_signer().seq(),
            end.is_writer().seq(),
        ),
        data: EndRebalanceIxData::as_buf().into(),
    }
}

/// Currently assumes that StartRebalance is the first ix
/// and EndRebalance is the last ix
fn to_inp(
    start: &StartArgs,
    mid: impl IntoIterator<Item = Instruction>,
    end: &Option<EndAccs>,
    ams: impl IntoIterator<Item = AccountMap>,
) -> (Vec<Instruction>, AccountMap) {
    let start_ix = to_start_ix(start);
    let end_ix_opt = end.as_ref().map(to_end_ix);
    let ixs: Vec<_> = once(start_ix).chain(mid).chain(end_ix_opt).collect();

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

    // is_rebalancing=false for both before and aft
    [ps_bef, ps_aft]
        .into_iter()
        .for_each(|p| assert!(!U8Bool(&p.is_rebalancing).to_bool()));

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
    const AMOUNT: u64 = 100_000;
    const CURR_EPOCH: u64 = 0;

    let (start_accs, am) = jupsol_o_wsol_i_fixture_accs();

    let start_args = StartArgs {
        out_lst_index: JUPSOL_FIXTURE_LST_IDX.try_into().unwrap(),
        inp_lst_index: WSOL_FIXTURE_LST_IDX.try_into().unwrap(),
        amount: AMOUNT,
        min_starting_out_lst: 0,
        max_starting_inp_lst: u64::MAX,
        accs: start_accs,
    };

    let out_calc = derive_svc_no_inf(&am, &start_accs.out_calc, CURR_EPOCH);
    let inp_calc = SvcAg::Wsol(WsolCalc);
    let [inp_reserves, out_reserves] = [
        start_accs.ix_prefix.inp_pool_reserves(),
        start_accs.ix_prefix.out_pool_reserves(),
    ]
    .map(|a| get_token_account_amount(&am[&(*a).into()].data));

    let RebalanceQuote { inp, out, .. } = quote_rebalance_exact_out(RebalanceQuoteArgs {
        amt: AMOUNT,
        inp_reserves,
        out_reserves,
        inp_mint: *start_accs.ix_prefix.inp_lst_mint(),
        out_mint: *start_accs.ix_prefix.out_lst_mint(),
        inp_calc,
        out_calc,
    })
    .unwrap();

    let (ixs, bef) = to_inp(
        &start_args,
        [create_transfer_ix(
            &NewTransferIxAccsBuilder::start()
                .with_auth(*start_accs.ix_prefix.rebalance_auth())
                .with_dst(*start_accs.ix_prefix.inp_pool_reserves())
                .with_src(DONOR_TOKEN_ACC_ADDR.to_bytes())
                .build(),
            inp,
        )],
        &Some(EndAccs::from_start(start_accs)),
        [
            am,
            once((
                DONOR_TOKEN_ACC_ADDR,
                mock_token_acc(raw_token_acc(
                    *start_accs.ix_prefix.inp_lst_mint(),
                    *start_accs.ix_prefix.rebalance_auth(),
                    inp,
                )),
            ))
            .collect(),
        ],
    );

    SVM.with(|svm| rebalance_test(svm, &bef, &ixs, &out_calc, &inp_calc, None::<ProgramError>));

    expect![[r#"
        (
            100000,
            111331,
        )
    "#]]
    .assert_debug_eq(&(out, inp));
}

#[test]
fn rebal_jupsol_o_wsol_i_fixture_missing_end() {
    const AMOUNT: u64 = 100_000;
    const INP_AMT: u64 = 111_331;
    const CURR_EPOCH: u64 = 0;

    let (start_accs, am) = jupsol_o_wsol_i_fixture_accs();

    let start_args = StartArgs {
        out_lst_index: JUPSOL_FIXTURE_LST_IDX.try_into().unwrap(),
        inp_lst_index: WSOL_FIXTURE_LST_IDX.try_into().unwrap(),
        amount: AMOUNT,
        min_starting_out_lst: 0,
        max_starting_inp_lst: u64::MAX,
        accs: start_accs,
    };

    let out_calc = derive_svc_no_inf(&am, &start_accs.out_calc, CURR_EPOCH);
    let inp_calc = SvcAg::Wsol(WsolCalc);

    let (ixs, bef) = to_inp(
        &start_args,
        [create_transfer_ix(
            &NewTransferIxAccsBuilder::start()
                .with_auth(*start_accs.ix_prefix.rebalance_auth())
                .with_dst(*start_accs.ix_prefix.inp_pool_reserves())
                .with_src(DONOR_TOKEN_ACC_ADDR.to_bytes())
                .build(),
            INP_AMT,
        )],
        // No EndRebalance
        &None,
        [
            am,
            once((
                DONOR_TOKEN_ACC_ADDR,
                mock_token_acc(raw_token_acc(
                    *start_accs.ix_prefix.inp_lst_mint(),
                    *start_accs.ix_prefix.rebalance_auth(),
                    INP_AMT,
                )),
            ))
            .collect(),
        ],
    );

    SVM.with(|svm| {
        rebalance_test(
            svm,
            &bef,
            &ixs,
            &out_calc,
            &inp_calc,
            Some(Inf1CtlCustomProgErr(Inf1CtlErr::NoSucceedingEndRebalance)),
        )
    });
}

#[test]
fn rebal_jupsol_o_wsol_i_fixture_wrong_end_mint() {
    const AMOUNT: u64 = 100_000;
    const INP_AMT: u64 = 111_331;
    const CURR_EPOCH: u64 = 0;

    let (start_accs, mut am) = jupsol_o_wsol_i_fixture_accs();

    let start_args = StartArgs {
        out_lst_index: JUPSOL_FIXTURE_LST_IDX.try_into().unwrap(),
        inp_lst_index: WSOL_FIXTURE_LST_IDX.try_into().unwrap(),
        amount: AMOUNT,
        min_starting_out_lst: 0,
        max_starting_inp_lst: u64::MAX,
        accs: start_accs,
    };

    let out_calc = derive_svc_no_inf(&am, &start_accs.out_calc, CURR_EPOCH);
    let inp_calc = SvcAg::Wsol(WsolCalc);

    // override end mint for EndAccs
    let mut end_accs = EndAccs::from_start(start_accs);
    let wrong_inp_lst_mint = Pubkey::new_unique();
    let inp_lst_mint_acc = am[&(*end_accs.ix_prefix.inp_lst_mint()).into()].clone();
    am.insert(wrong_inp_lst_mint, inp_lst_mint_acc);
    end_accs
        .ix_prefix
        .set_inp_lst_mint(wrong_inp_lst_mint.to_bytes());

    let (ixs, bef) = to_inp(
        &start_args,
        [create_transfer_ix(
            &NewTransferIxAccsBuilder::start()
                .with_auth(*start_accs.ix_prefix.rebalance_auth())
                .with_dst(*start_accs.ix_prefix.inp_pool_reserves())
                .with_src(DONOR_TOKEN_ACC_ADDR.to_bytes())
                .build(),
            INP_AMT,
        )],
        &Some(end_accs),
        [
            am,
            once((
                DONOR_TOKEN_ACC_ADDR,
                mock_token_acc(raw_token_acc(
                    *start_accs.ix_prefix.inp_lst_mint(),
                    *start_accs.ix_prefix.rebalance_auth(),
                    INP_AMT,
                )),
            ))
            .collect(),
        ],
    );

    SVM.with(|svm| {
        rebalance_test(
            svm,
            &bef,
            &ixs,
            &out_calc,
            &inp_calc,
            Some(Inf1CtlCustomProgErr(Inf1CtlErr::NoSucceedingEndRebalance)),
        )
    });
}

#[test]
fn rebal_jupsol_o_wsol_i_fixture_insufficient_transfer() {
    const AMOUNT: u64 = 100_000;
    const CURR_EPOCH: u64 = 0;

    let (start_accs, am) = jupsol_o_wsol_i_fixture_accs();

    let start_args = StartArgs {
        out_lst_index: JUPSOL_FIXTURE_LST_IDX.try_into().unwrap(),
        inp_lst_index: WSOL_FIXTURE_LST_IDX.try_into().unwrap(),
        amount: AMOUNT,
        min_starting_out_lst: 0,
        max_starting_inp_lst: u64::MAX,
        accs: start_accs,
    };

    let out_calc = derive_svc_no_inf(&am, &start_accs.out_calc, CURR_EPOCH);
    let inp_calc = SvcAg::Wsol(WsolCalc);
    let [inp_reserves, out_reserves] = [
        start_accs.ix_prefix.inp_pool_reserves(),
        start_accs.ix_prefix.out_pool_reserves(),
    ]
    .map(|a| get_token_account_amount(&am[&(*a).into()].data));

    let RebalanceQuote { inp, .. } = quote_rebalance_exact_out(RebalanceQuoteArgs {
        amt: AMOUNT,
        inp_reserves,
        out_reserves,
        inp_mint: *start_accs.ix_prefix.inp_lst_mint(),
        out_mint: *start_accs.ix_prefix.out_lst_mint(),
        inp_calc,
        out_calc,
    })
    .unwrap();
    // transfer 1 less than required
    let insuff = inp - 1;

    let (ixs, bef) = to_inp(
        &start_args,
        [create_transfer_ix(
            &NewTransferIxAccsBuilder::start()
                .with_auth(*start_accs.ix_prefix.rebalance_auth())
                .with_dst(*start_accs.ix_prefix.inp_pool_reserves())
                .with_src(DONOR_TOKEN_ACC_ADDR.to_bytes())
                .build(),
            insuff,
        )],
        &Some(EndAccs::from_start(start_accs)),
        [
            am,
            once((
                DONOR_TOKEN_ACC_ADDR,
                mock_token_acc(raw_token_acc(
                    *start_accs.ix_prefix.inp_lst_mint(),
                    *start_accs.ix_prefix.rebalance_auth(),
                    insuff,
                )),
            ))
            .collect(),
        ],
    );

    SVM.with(|svm| {
        rebalance_test(
            svm,
            &bef,
            &ixs,
            &out_calc,
            &inp_calc,
            Some(Inf1CtlCustomProgErr(Inf1CtlErr::PoolWouldLoseSolValue)),
        )
    });
}

enum SlippageDir {
    MinOut,
    MaxInp,
}

fn rebal_jupsol_o_wsol_i_fixture_slippage_violated(dir: SlippageDir) {
    const AMOUNT: u64 = 100_000;
    const INP_AMT: u64 = 111_331;
    const CURR_EPOCH: u64 = 0;

    let (start_accs, am) = jupsol_o_wsol_i_fixture_accs();

    let [inp_reserves, out_reserves] = [
        start_accs.ix_prefix.inp_pool_reserves(),
        start_accs.ix_prefix.out_pool_reserves(),
    ]
    .map(|a| get_token_account_amount(&am[&(*a).into()].data));

    // set just exceeding the slippage limit
    let [min_starting_out_lst, max_starting_inp_lst] = match dir {
        SlippageDir::MaxInp => [0, inp_reserves - 1],
        SlippageDir::MinOut => [out_reserves + 1, u64::MAX],
    };

    let start_args = StartArgs {
        out_lst_index: JUPSOL_FIXTURE_LST_IDX.try_into().unwrap(),
        inp_lst_index: WSOL_FIXTURE_LST_IDX.try_into().unwrap(),
        amount: AMOUNT,
        min_starting_out_lst,
        max_starting_inp_lst,
        accs: start_accs,
    };

    let out_calc = derive_svc_no_inf(&am, &start_accs.out_calc, CURR_EPOCH);
    let inp_calc = SvcAg::Wsol(WsolCalc);

    let (ixs, bef) = to_inp(
        &start_args,
        [create_transfer_ix(
            &NewTransferIxAccsBuilder::start()
                .with_auth(*start_accs.ix_prefix.rebalance_auth())
                .with_dst(*start_accs.ix_prefix.inp_pool_reserves())
                .with_src(DONOR_TOKEN_ACC_ADDR.to_bytes())
                .build(),
            INP_AMT,
        )],
        &Some(EndAccs::from_start(start_accs)),
        [
            am,
            once((
                DONOR_TOKEN_ACC_ADDR,
                mock_token_acc(raw_token_acc(
                    *start_accs.ix_prefix.inp_lst_mint(),
                    *start_accs.ix_prefix.rebalance_auth(),
                    INP_AMT,
                )),
            ))
            .collect(),
        ],
    );

    SVM.with(|svm| {
        rebalance_test(
            svm,
            &bef,
            &ixs,
            &out_calc,
            &inp_calc,
            Some(Inf1CtlCustomProgErr(Inf1CtlErr::SlippageToleranceExceeded)),
        )
    });
}

#[test]
fn rebal_jupsol_o_wsol_i_fixture_slippage_min_out_violated() {
    rebal_jupsol_o_wsol_i_fixture_slippage_violated(SlippageDir::MinOut);
}

#[test]
fn rebal_jupsol_o_wsol_i_fixture_slippage_max_inp_violated() {
    rebal_jupsol_o_wsol_i_fixture_slippage_violated(SlippageDir::MaxInp);
}

#[test]
fn rebal_jupsol_o_wsol_i_fixture_pool_already_rebalancing() {
    const AMOUNT: u64 = 100_000;
    const INP_AMT: u64 = 111_331;
    const CURR_EPOCH: u64 = 0;

    let (start_accs, am) = jupsol_o_wsol_i_fixture_accs();

    let start_args = StartArgs {
        out_lst_index: JUPSOL_FIXTURE_LST_IDX.try_into().unwrap(),
        inp_lst_index: WSOL_FIXTURE_LST_IDX.try_into().unwrap(),
        amount: AMOUNT,
        min_starting_out_lst: 0,
        max_starting_inp_lst: u64::MAX,
        accs: start_accs,
    };

    let out_calc = derive_svc_no_inf(&am, &start_accs.out_calc, CURR_EPOCH);
    let inp_calc = SvcAg::Wsol(WsolCalc);

    let (ixs, bef) = to_inp(
        &start_args,
        [
            create_transfer_ix(
                &NewTransferIxAccsBuilder::start()
                    .with_auth(*start_accs.ix_prefix.rebalance_auth())
                    .with_dst(*start_accs.ix_prefix.inp_pool_reserves())
                    .with_src(DONOR_TOKEN_ACC_ADDR.to_bytes())
                    .build(),
                INP_AMT,
            ),
            // create another StartRebalanceIx
            // in the middle before the end
            to_start_ix(&start_args),
        ],
        &Some(EndAccs::from_start(start_accs)),
        [
            am,
            once((
                DONOR_TOKEN_ACC_ADDR,
                mock_token_acc(raw_token_acc(
                    *start_accs.ix_prefix.inp_lst_mint(),
                    *start_accs.ix_prefix.rebalance_auth(),
                    INP_AMT,
                )),
            ))
            .collect(),
        ],
    );

    SVM.with(|svm| {
        rebalance_test(
            svm,
            &bef,
            &ixs,
            &out_calc,
            &inp_calc,
            Some(Inf1CtlCustomProgErr(Inf1CtlErr::PoolRebalancing)),
        )
    });
}

#[test]
fn rebal_jupsol_o_wsol_i_fixture_pool_unauthorized() {
    const AMOUNT: u64 = 100_000;
    const INP_AMT: u64 = 111_331;
    const CURR_EPOCH: u64 = 0;

    let (mut start_accs, mut am) = jupsol_o_wsol_i_fixture_accs();

    // create unauthorized rebalance auth acc
    let unauth = Pubkey::new_unique();
    start_accs.ix_prefix.set_rebalance_auth(unauth.to_bytes());
    am.insert(unauth, mock_sys_acc(1_000_000_000));

    let start_args = StartArgs {
        out_lst_index: JUPSOL_FIXTURE_LST_IDX.try_into().unwrap(),
        inp_lst_index: WSOL_FIXTURE_LST_IDX.try_into().unwrap(),
        amount: AMOUNT,
        min_starting_out_lst: 0,
        max_starting_inp_lst: u64::MAX,
        accs: start_accs,
    };

    let out_calc = derive_svc_no_inf(&am, &start_accs.out_calc, CURR_EPOCH);
    let inp_calc = SvcAg::Wsol(WsolCalc);

    let (ixs, bef) = to_inp(
        &start_args,
        [create_transfer_ix(
            &NewTransferIxAccsBuilder::start()
                .with_auth(*start_accs.ix_prefix.rebalance_auth())
                .with_dst(*start_accs.ix_prefix.inp_pool_reserves())
                .with_src(DONOR_TOKEN_ACC_ADDR.to_bytes())
                .build(),
            INP_AMT,
        )],
        &Some(EndAccs::from_start(start_accs)),
        [
            am,
            once((
                DONOR_TOKEN_ACC_ADDR,
                mock_token_acc(raw_token_acc(
                    *start_accs.ix_prefix.inp_lst_mint(),
                    *start_accs.ix_prefix.rebalance_auth(),
                    INP_AMT,
                )),
            ))
            .collect(),
        ],
    );

    SVM.with(|svm| {
        rebalance_test(
            svm,
            &bef,
            &ixs,
            &out_calc,
            &inp_calc,
            Some(INVALID_ARGUMENT),
        )
    });
}
