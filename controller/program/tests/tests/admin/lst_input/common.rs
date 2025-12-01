use inf1_ctl_jiminy::{
    accounts::{lst_state_list::LstStatePackedList, pool_state::PoolStateV2},
    instructions::admin::lst_input::{
        NewSetLstInputIxAccsBuilder, SetLstInputIxAccsBuilder, SetLstInputIxKeysOwned,
        SET_LST_INPUT_IX_ACCS_IDX_LST_MINT,
    },
    keys::{LST_STATE_LIST_ID, POOL_STATE_ID},
    typedefs::{lst_state::LstState, u8bool::U8Bool},
};
use inf1_test_utils::{
    acc_bef_aft, any_lst_state, any_pool_state_v2, assert_diffs_lst_state_list,
    assert_jiminy_prog_err, distinct_idxs, idx_oob, list_sample_flat_map, lst_state_list_account,
    mock_mint, mock_sys_acc, mollusk_exec, pool_state_v2_account,
    pool_state_v2_u8_bools_normal_strat, raw_mint, AccountMap, AnyLstStateArgs, Diff, ExecResult,
    LstStateArgs, LstStateData, LstStateListChanges, PoolStateV2FtaStrat,
};
use jiminy_cpi::program_error::ProgramError;
use mollusk_svm::result::ProgramResult;
use proptest::{collection::vec, prelude::*};
use solana_instruction::Instruction;

use crate::common::{MAX_LST_STATES, SVM};

pub fn set_lst_input_test(
    expected_is_input_disabled: bool,
    ix: Instruction,
    bef: &AccountMap,
    expected_err: Option<impl Into<ProgramError>>,
) {
    let mint = ix.accounts[SET_LST_INPUT_IX_ACCS_IDX_LST_MINT].pubkey;
    let (aft, ExecResult { program_result, .. }) = SVM.with(|svm| mollusk_exec(svm, &[ix], bef));

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
            let iid_bef = U8Bool(
                &list_bef
                    .iter()
                    .find(|s| s.mint == mint.to_bytes())
                    .unwrap()
                    .is_input_disabled,
            )
            .to_bool();
            assert_diffs_lst_state_list(
                LstStateListChanges::new(&list_bef)
                    .with_diff_by_mint(
                        mint.as_array(),
                        LstStateArgs {
                            // not strict: Enable/DisableLstInput ixs are idempotent
                            is_input_disabled: Diff::Changed(iid_bef, expected_is_input_disabled),
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
pub fn set_lst_input_partial_keys() -> SetLstInputIxAccsBuilder<[u8; 32], false, false, true, true>
{
    NewSetLstInputIxAccsBuilder::start()
        .with_lst_state_list(LST_STATE_LIST_ID)
        .with_pool_state(POOL_STATE_ID)
}

pub fn set_lst_input_test_accs(
    keys: SetLstInputIxKeysOwned,
    pool: PoolStateV2,
    lst_state_list: Vec<LstState>,
) -> AccountMap {
    // dont care abt lamports, shouldnt affect anything
    const LAMPORTS: u64 = 1_000_000_000;
    let accs = NewSetLstInputIxAccsBuilder::start()
        .with_admin(mock_sys_acc(LAMPORTS))
        // mint parameters do not affect this instruction
        .with_lst_mint(mock_mint(raw_mint(None, None, 0, 0)))
        .with_lst_state_list(lst_state_list_account(
            lst_state_list
                .into_iter()
                .flat_map(|l| *l.as_acc_data_arr())
                .collect(),
        ))
        .with_pool_state(pool_state_v2_account(pool))
        .build();
    keys.0.into_iter().map(Into::into).zip(accs.0).collect()
}

pub type ToInpTup = (
    SetLstInputIxKeysOwned,
    usize,
    PoolStateV2,
    Vec<LstStateData>,
);

/// Given a PoolState, generate a set of Dis/EnableLstInput ix args that will result in correct execution
/// - admin is set to correct admin
/// - randomly generated LstStateList of at least 1 element. is_input_disabled can be anything
/// - mint and idx are set correctly to a random sample from this list
pub fn correct_to_inp_strat(ps: PoolStateV2) -> impl Strategy<Value = ToInpTup> {
    (
        Just(ps),
        vec(
            any_lst_state(AnyLstStateArgs::default(), None),
            1..=MAX_LST_STATES,
        )
        .prop_flat_map(list_sample_flat_map),
    )
        .prop_map(|(ps, (idx, s, l))| {
            (
                set_lst_input_partial_keys()
                    .with_lst_mint(s.lst_state.mint)
                    .with_admin(ps.admin)
                    .build(),
                idx,
                ps,
                l,
            )
        })
}

/// Given a PoolState, generate a set of set lst input ix args that will result in
/// an unauthorized err when executing Dis/EnableLstInput
pub fn unauthorized_to_inp_strat(ps: PoolStateV2) -> impl Strategy<Value = ToInpTup> {
    (
        any::<[u8; 32]>().prop_filter("", move |pk| *pk != ps.admin),
        Just(ps),
        vec(
            any_lst_state(AnyLstStateArgs::default(), None),
            1..=MAX_LST_STATES,
        )
        .prop_flat_map(list_sample_flat_map),
    )
        .prop_map(|(unauth, ps, (idx, s, l))| {
            (
                set_lst_input_partial_keys()
                    .with_lst_mint(s.lst_state.mint)
                    .with_admin(unauth)
                    .build(),
                idx,
                ps,
                l,
            )
        })
}

/// Generate a set of Dis/EnableLstInput ix args that has lst index ix arg
/// out of bounds but would otherwise succeed
pub fn lst_idx_oob_to_inp_strat() -> impl Strategy<Value = ToInpTup> {
    vec(
        any_lst_state(AnyLstStateArgs::default(), None),
        0..=MAX_LST_STATES,
    )
    .prop_flat_map(|l| {
        (
            idx_oob(l.len()),
            any::<[u8; 32]>(),
            Just(l),
            any_pool_state_v2(PoolStateV2FtaStrat {
                u8_bools: pool_state_v2_u8_bools_normal_strat(),
                ..Default::default()
            }),
        )
    })
    .prop_map(|(idx, rand_mint, list, ps)| {
        (
            set_lst_input_partial_keys()
                .with_admin(ps.admin)
                // mint is random bec ix should fail
                // before we get to verifying its identity
                .with_lst_mint(rand_mint)
                .build(),
            idx,
            ps,
            list,
        )
    })
}

/// like [`lst_idx_oob_to_inp_strat`] but the error is that the provided mint
/// does not match the LstState at the entry of the given index
pub fn lst_idx_mismatch_to_inp_strat() -> impl Strategy<Value = ToInpTup> {
    vec(
        any_lst_state(AnyLstStateArgs::default(), None),
        2..=MAX_LST_STATES, // need at least 2 for distinct
    )
    .prop_flat_map(|l| {
        (
            distinct_idxs(l.len()),
            Just(l),
            any_pool_state_v2(PoolStateV2FtaStrat {
                u8_bools: pool_state_v2_u8_bools_normal_strat(),
                ..Default::default()
            }),
        )
    })
    .prop_map(|((x, y), list, ps)| {
        (
            set_lst_input_partial_keys()
                .with_admin(ps.admin)
                .with_lst_mint(list[x].lst_state.mint)
                .build(),
            y,
            ps,
            list,
        )
    })
}
