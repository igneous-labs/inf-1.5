use inf1_ctl_jiminy::{
    accounts::{lst_state_list::LstStatePackedList, pool_state::PoolState},
    instructions::admin::lst_input::{
        disable::{
            DisableLstInputIxData, DisableLstInputIxKeysOwned, NewDisableLstInputIxAccsBuilder,
            DISABLE_LST_INPUT_IX_IS_SIGNER, DISABLE_LST_INPUT_IX_IS_WRITER,
        },
        NewSetLstInputIxAccsBuilder, SetLstInputIxAccsBuilder, SET_LST_INPUT_IX_ACCS_IDX_LST_MINT,
    },
    keys::{LST_STATE_LIST_ID, POOL_STATE_ID},
    typedefs::{lst_state::LstState, u8bool::U8Bool},
};
use inf1_test_utils::{
    acc_bef_aft, assert_diffs_lst_state_list, assert_jiminy_prog_err, dedup_accounts,
    gen_pool_state, keys_signer_writable_to_metas, lst_state_list_account, mock_mint, mock_sys_acc,
    pool_state_account, raw_mint, Diff, GenPoolStateArgs, LstStateArgs, LstStateListChanges,
    PkAccountTup, PoolStatePks,
};
use jiminy_cpi::program_error::ProgramError;
use mollusk_svm::result::{InstructionResult, ProgramResult};
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

use crate::common::SVM;

fn disable_lst_input_ix(keys: DisableLstInputIxKeysOwned, idx: usize) -> Instruction {
    let accounts = keys_signer_writable_to_metas(
        keys.0.iter(),
        DISABLE_LST_INPUT_IX_IS_SIGNER.0.iter(),
        DISABLE_LST_INPUT_IX_IS_WRITER.0.iter(),
    );
    Instruction {
        program_id: Pubkey::new_from_array(inf1_ctl_jiminy::ID),
        accounts,
        data: DisableLstInputIxData::new(idx.try_into().unwrap())
            .as_buf()
            .into(),
    }
}

fn disable_lst_input_test_accs(
    keys: DisableLstInputIxKeysOwned,
    pool: PoolState,
    lst_state_list: Vec<LstState>,
) -> Vec<PkAccountTup> {
    // dont care abt lamports, shouldnt affect anything
    const LAMPORTS: u64 = 1_000_000_000;
    let accs = NewDisableLstInputIxAccsBuilder::start()
        .with_admin(mock_sys_acc(LAMPORTS))
        // mint parameters do not affect this instruction
        .with_lst_mint(mock_mint(raw_mint(None, None, 0, 0)))
        .with_lst_state_list(lst_state_list_account(
            lst_state_list
                .into_iter()
                .flat_map(|l| *l.as_acc_data_arr())
                .collect(),
        ))
        .with_pool_state(pool_state_account(pool))
        .build();
    let mut res = keys.0.into_iter().map(Into::into).zip(accs.0).collect();
    dedup_accounts(&mut res);
    res
}

fn disable_lst_input_test(
    ix: &Instruction,
    bef: &[PkAccountTup],
    expected_err: Option<impl Into<ProgramError>>,
) {
    let InstructionResult {
        program_result,
        resulting_accounts: aft,
        ..
    } = SVM.with(|svm| svm.process_instruction(ix, bef));

    let [list_bef, list_aft] = acc_bef_aft(&LST_STATE_LIST_ID.into(), bef, &aft).map(|a| {
        LstStatePackedList::of_acc_data(&a.data)
            .unwrap()
            .0
            .iter()
            .map(|s| s.into_lst_state())
            .collect::<Vec<_>>()
    });

    match expected_err {
        None => {
            assert_eq!(program_result, ProgramResult::Success);
            let mint = ix.accounts[SET_LST_INPUT_IX_ACCS_IDX_LST_MINT].pubkey;
            let iid_bef = U8Bool(
                &list_bef
                    .iter()
                    .find(|s| s.mint == mint.to_bytes())
                    .unwrap()
                    .is_input_disabled,
            )
            .as_bool();
            assert_diffs_lst_state_list(
                LstStateListChanges::new(&list_bef)
                    .with_diff_by_mint(
                        mint.as_array(),
                        LstStateArgs {
                            // not strict: ix is idempotent
                            is_input_disabled: Diff::Changed(iid_bef, true),
                            ..Default::default()
                        },
                    )
                    .build(),
                list_bef,
                list_aft,
            );
        }
        Some(e) => {
            assert_jiminy_prog_err(&program_result, e);
        }
    }
}

/// Missing admin and mint
fn partial_keys() -> SetLstInputIxAccsBuilder<[u8; 32], false, false, true, true> {
    NewSetLstInputIxAccsBuilder::start()
        .with_lst_state_list(LST_STATE_LIST_ID)
        .with_pool_state(POOL_STATE_ID)
}

#[test]
fn disable_lst_input_correct_basic() {
    let [admin, mint] = core::array::from_fn(|i| [69 + u8::try_from(i).unwrap(); 32]);
    let pool = gen_pool_state(GenPoolStateArgs {
        pks: PoolStatePks::default().with_admin(admin),
        ..Default::default()
    });
    let lst_state_list = vec![LstState {
        mint,
        is_input_disabled: 0,
        pool_reserves_bump: 255,
        protocol_fee_accumulator_bump: 255,
        padding: [0; 5],
        sol_value: 0,
        sol_value_calculator: [3u8; 32],
    }];
    let keys = partial_keys().with_admin(admin).with_lst_mint(mint).build();
    disable_lst_input_test(
        &disable_lst_input_ix(keys, 0),
        &disable_lst_input_test_accs(keys, pool, lst_state_list),
        Option::<ProgramError>::None,
    );
}
