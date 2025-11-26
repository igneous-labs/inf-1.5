use inf1_ctl_jiminy::{
    accounts::pool_state::{PoolState, PoolStatePacked},
    instructions::protocol_fee::set_protocol_fee_beneficiary::{
        NewSetProtocolFeeBeneficiaryIxAccsBuilder, SetProtocolFeeBeneficiaryIxData,
        SetProtocolFeeBeneficiaryIxKeysOwned, SET_PROTOCOL_FEE_BENEFICIARY_IX_ACCS_IDX_CURR,
        SET_PROTOCOL_FEE_BENEFICIARY_IX_ACCS_IDX_NEW, SET_PROTOCOL_FEE_BENEFICIARY_IX_IS_SIGNER,
        SET_PROTOCOL_FEE_BENEFICIARY_IX_IS_WRITER,
    },
    keys::POOL_STATE_ID,
    ID,
};
use inf1_test_utils::{
    acc_bef_aft, any_normal_pk, any_pool_state, assert_diffs_pool_state, assert_jiminy_prog_err,
    gen_pool_state, keys_signer_writable_to_metas, mock_sys_acc, mollusk_exec, pool_state_account,
    silence_mollusk_logs, AccountMap, Diff, DiffsPoolStateArgs, GenPoolStateArgs, PoolStatePks,
};
use jiminy_cpi::program_error::{ProgramError, INVALID_ARGUMENT, MISSING_REQUIRED_SIGNATURE};
use mollusk_svm::result::{InstructionResult, ProgramResult};
use proptest::prelude::*;
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

use crate::common::SVM;

fn set_protocol_fee_beneficiary_ix(keys: SetProtocolFeeBeneficiaryIxKeysOwned) -> Instruction {
    let accounts = keys_signer_writable_to_metas(
        keys.0.iter(),
        SET_PROTOCOL_FEE_BENEFICIARY_IX_IS_SIGNER.0.iter(),
        SET_PROTOCOL_FEE_BENEFICIARY_IX_IS_WRITER.0.iter(),
    );
    Instruction {
        program_id: Pubkey::new_from_array(ID),
        accounts,
        data: SetProtocolFeeBeneficiaryIxData::as_buf().into(),
    }
}

fn set_protocol_fee_beneficiary_ix_test_accs(
    keys: SetProtocolFeeBeneficiaryIxKeysOwned,
    pool: PoolState,
) -> AccountMap {
    // dont care abt lamports, shouldnt affect anything
    const LAMPORTS: u64 = 1_000_000_000;
    let accs = NewSetProtocolFeeBeneficiaryIxAccsBuilder::start()
        .with_curr(mock_sys_acc(LAMPORTS))
        .with_new(mock_sys_acc(LAMPORTS))
        .with_pool_state(pool_state_account(pool))
        .build();
    keys.0.into_iter().map(Into::into).zip(accs.0).collect()
}

/// Returns `pool_state.protocol_fee_beneficiary` at the end of ix
fn set_protocol_fee_beneficiary_test(
    ix: &Instruction,
    bef: &AccountMap,
    expected_err: Option<impl Into<ProgramError>>,
) -> [u8; 32] {
    let (
        bef,
        InstructionResult {
            program_result,
            resulting_accounts,
            ..
        },
    ) = SVM.with(|svm| mollusk_exec(svm, ix, bef));
    let aft: AccountMap = resulting_accounts.into_iter().collect();

    let [pool_state_bef, pool_state_aft] =
        acc_bef_aft(&POOL_STATE_ID.into(), &bef, &aft).map(|a| {
            PoolStatePacked::of_acc_data(&a.data)
                .unwrap()
                .into_pool_state()
        });

    let curr_ben = pool_state_bef.protocol_fee_beneficiary;
    let expected_new_ben = ix.accounts[SET_PROTOCOL_FEE_BENEFICIARY_IX_ACCS_IDX_NEW].pubkey;

    match expected_err {
        None => {
            assert_eq!(program_result, ProgramResult::Success);
            assert_diffs_pool_state(
                &DiffsPoolStateArgs {
                    pks: PoolStatePks::default().with_protocol_fee_beneficiary(Diff::Changed(
                        curr_ben,
                        expected_new_ben.to_bytes(),
                    )),
                    ..Default::default()
                },
                &pool_state_bef,
                &pool_state_aft,
            );
        }
        Some(e) => {
            assert_jiminy_prog_err(&program_result, e);
        }
    }

    pool_state_aft.protocol_fee_beneficiary
}

#[test]
fn set_protocol_fee_beneficiary_test_correct_basic() {
    let [curr_ben, new_ben] = core::array::from_fn(|i| [u8::try_from(i).unwrap(); 32]);
    let pool = gen_pool_state(GenPoolStateArgs {
        pks: PoolStatePks::default().with_protocol_fee_beneficiary(curr_ben),
        version: 1,
        ..Default::default()
    });
    let keys = NewSetProtocolFeeBeneficiaryIxAccsBuilder::start()
        .with_new(new_ben)
        .with_curr(curr_ben)
        .with_pool_state(POOL_STATE_ID)
        .build();
    let ret = set_protocol_fee_beneficiary_test(
        &set_protocol_fee_beneficiary_ix(keys),
        &set_protocol_fee_beneficiary_ix_test_accs(keys, pool),
        Option::<ProgramError>::None,
    );
    assert_eq!(ret, new_ben);
}

fn correct_strat() -> impl Strategy<Value = (Instruction, AccountMap)> {
    (any_normal_pk(), any_pool_state(Default::default()))
        .prop_map(|(new_ben, ps)| {
            (
                NewSetProtocolFeeBeneficiaryIxAccsBuilder::start()
                    .with_new(new_ben)
                    .with_curr(ps.protocol_fee_beneficiary)
                    .with_pool_state(POOL_STATE_ID)
                    .build(),
                ps,
            )
        })
        .prop_map(|(k, ps)| {
            (
                set_protocol_fee_beneficiary_ix(k),
                set_protocol_fee_beneficiary_ix_test_accs(k, ps),
            )
        })
}

fn unauthorized_strat() -> impl Strategy<Value = (Instruction, AccountMap)> {
    (any_normal_pk(), any_pool_state(Default::default()))
        .prop_flat_map(|(new_ben, ps)| {
            (
                any::<[u8; 32]>().prop_filter("", move |pk| *pk != ps.protocol_fee_beneficiary),
                Just(new_ben),
                Just(ps),
            )
        })
        .prop_map(|(wrong_curr_ben, new_ben, ps)| {
            (
                NewSetProtocolFeeBeneficiaryIxAccsBuilder::start()
                    .with_new(new_ben)
                    .with_curr(wrong_curr_ben)
                    .with_pool_state(POOL_STATE_ID)
                    .build(),
                ps,
            )
        })
        .prop_map(|(k, ps)| {
            (
                set_protocol_fee_beneficiary_ix(k),
                set_protocol_fee_beneficiary_ix_test_accs(k, ps),
            )
        })
}

fn missing_sig_strat() -> impl Strategy<Value = (Instruction, AccountMap)> {
    correct_strat().prop_map(|(mut ix, accs)| {
        ix.accounts[SET_PROTOCOL_FEE_BENEFICIARY_IX_ACCS_IDX_CURR].is_signer = false;
        (ix, accs)
    })
}

proptest! {
    #[test]
    fn set_protocol_fee_beneficiary_correct_pt(
        (ix, bef) in correct_strat(),
    ) {
        silence_mollusk_logs();
        set_protocol_fee_beneficiary_test(&ix, &bef, Option::<ProgramError>::None);
    }
}

proptest! {
    #[test]
    fn set_protocol_fee_beneficiary_unauthorized_pt(
        (ix, bef) in unauthorized_strat(),
    ) {
        silence_mollusk_logs();
        set_protocol_fee_beneficiary_test(&ix, &bef, Some(INVALID_ARGUMENT));
    }
}

proptest! {
    #[test]
    fn set_protocol_fee_beneficiary_missing_sig_pt(
        (ix, bef) in missing_sig_strat(),
    ) {
        silence_mollusk_logs();
        set_protocol_fee_beneficiary_test(&ix, &bef, Some(MISSING_REQUIRED_SIGNATURE));
    }
}
