use inf1_ctl_jiminy::{
    accounts::pool_state::PoolState,
    err::Inf1CtlErr,
    instructions::admin::lst_input::{
        disable::{
            DisableLstInputIxData, DisableLstInputIxKeysOwned, DISABLE_LST_INPUT_IX_IS_SIGNER,
            DISABLE_LST_INPUT_IX_IS_WRITER,
        },
        SetLstInputIxKeysOwned, SET_LST_INPUT_IX_ACCS_IDX_ADMIN,
    },
    program_err::Inf1CtlCustomProgErr,
    typedefs::lst_state::LstState,
};
use inf1_test_utils::{
    any_pool_state, gen_pool_state, keys_signer_writable_to_metas, silence_mollusk_logs,
    AccountMap, AnyPoolStateArgs, GenPoolStateArgs, LstStateData, PoolStateBools, PoolStatePks,
};
use jiminy_cpi::program_error::{ProgramError, INVALID_ARGUMENT, MISSING_REQUIRED_SIGNATURE};
use proptest::prelude::*;
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

use crate::tests::admin::lst_input::common::{
    correct_to_inp_strat, lst_idx_mismatch_to_inp_strat, lst_idx_oob_to_inp_strat,
    set_lst_input_partial_keys, set_lst_input_test, set_lst_input_test_accs,
    unauthorized_to_inp_strat,
};

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

fn disable_lst_input_test(
    ix: &Instruction,
    bef: &AccountMap,
    expected_err: Option<impl Into<ProgramError>>,
) {
    set_lst_input_test(true, ix, bef, expected_err);
}

#[test]
fn disable_lst_input_correct_basic() {
    let [admin, mint] = core::array::from_fn(|i| [69 + u8::try_from(i).unwrap(); 32]);
    let pool = gen_pool_state(GenPoolStateArgs {
        pks: PoolStatePks::default().with_admin(admin),
        version: 1,
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
    let keys = set_lst_input_partial_keys()
        .with_admin(admin)
        .with_lst_mint(mint)
        .build();
    disable_lst_input_test(
        &disable_lst_input_ix(keys, 0),
        &set_lst_input_test_accs(keys, pool, lst_state_list),
        Option::<ProgramError>::None,
    );
}

fn to_inp(
    (keys, idx, pool, lst_state_list): (
        SetLstInputIxKeysOwned,
        usize,
        PoolState,
        Vec<LstStateData>,
    ),
) -> (Instruction, AccountMap) {
    (
        disable_lst_input_ix(keys, idx),
        set_lst_input_test_accs(
            keys,
            pool,
            lst_state_list.into_iter().map(|l| l.lst_state).collect(),
        ),
    )
}

fn correct_strat() -> impl Strategy<Value = (Instruction, AccountMap)> {
    any_pool_state(AnyPoolStateArgs {
        bools: PoolStateBools::normal(),
        ..Default::default()
    })
    .prop_flat_map(correct_to_inp_strat)
    .prop_map(to_inp)
}

proptest! {
    #[test]
    fn disable_lst_input_correct_pt(
        (ix, bef) in correct_strat(),
    ) {
        silence_mollusk_logs();
        disable_lst_input_test(&ix, &bef, Option::<ProgramError>::None);
    }
}

fn unauthorized_strat() -> impl Strategy<Value = (Instruction, AccountMap)> {
    any_pool_state(AnyPoolStateArgs {
        bools: PoolStateBools::normal(),
        ..Default::default()
    })
    .prop_flat_map(unauthorized_to_inp_strat)
    .prop_map(to_inp)
}

proptest! {
    #[test]
    fn disable_lst_input_unauth_pt(
        (ix, bef) in unauthorized_strat(),
    ) {
        silence_mollusk_logs();
        disable_lst_input_test(&ix, &bef, Some(INVALID_ARGUMENT));
    }
}

fn missing_sig_strat() -> impl Strategy<Value = (Instruction, AccountMap)> {
    correct_strat().prop_map(|(mut ix, accs)| {
        ix.accounts[SET_LST_INPUT_IX_ACCS_IDX_ADMIN].is_signer = false;
        (ix, accs)
    })
}

proptest! {
    #[test]
    fn disable_lst_input_missing_sig_pt(
        (ix, bef) in missing_sig_strat(),
    ) {
        silence_mollusk_logs();
        disable_lst_input_test(&ix, &bef, Some(MISSING_REQUIRED_SIGNATURE));
    }
}

fn rebalancing_strat() -> impl Strategy<Value = (Instruction, AccountMap)> {
    any_pool_state(AnyPoolStateArgs {
        bools: PoolStateBools::normal().with_is_rebalancing(Some(Just(true).boxed())),
        ..Default::default()
    })
    .prop_flat_map(correct_to_inp_strat)
    .prop_map(to_inp)
}

proptest! {
    #[test]
    fn disable_lst_input_rebalancing_pt(
        (ix, bef) in rebalancing_strat(),
    ) {
        silence_mollusk_logs();
        disable_lst_input_test(
            &ix,
            &bef,
            Some(Inf1CtlCustomProgErr(Inf1CtlErr::PoolRebalancing))
        );
    }
}

fn pool_disabled_strat() -> impl Strategy<Value = (Instruction, AccountMap)> {
    any_pool_state(AnyPoolStateArgs {
        bools: PoolStateBools::normal().with_is_disabled(Some(Just(true).boxed())),
        ..Default::default()
    })
    .prop_flat_map(correct_to_inp_strat)
    .prop_map(to_inp)
}

proptest! {
    #[test]
    fn disable_lst_input_pool_disabled_pt(
        (ix, bef) in pool_disabled_strat(),
    ) {
        silence_mollusk_logs();
        disable_lst_input_test(
            &ix,
            &bef,
            Some(Inf1CtlCustomProgErr(Inf1CtlErr::PoolDisabled))
        );
    }
}

fn lst_idx_oob_strat() -> impl Strategy<Value = (Instruction, AccountMap)> {
    lst_idx_oob_to_inp_strat().prop_map(to_inp)
}

proptest! {
    #[test]
    fn disable_lst_input_idx_oob(
        (ix, bef) in lst_idx_oob_strat(),
    ) {
        silence_mollusk_logs();
        disable_lst_input_test(
            &ix,
            &bef,
            Some(Inf1CtlCustomProgErr(Inf1CtlErr::InvalidLstIndex))
        );
    }
}

fn lst_idx_mismatch_strat() -> impl Strategy<Value = (Instruction, AccountMap)> {
    lst_idx_mismatch_to_inp_strat().prop_map(to_inp)
}

proptest! {
    #[test]
    fn disable_lst_input_idx_mismatch_pt(
        (ix, bef) in lst_idx_mismatch_strat(),
    ) {
        silence_mollusk_logs();
        disable_lst_input_test(&ix, &bef, Some(INVALID_ARGUMENT));
    }
}
