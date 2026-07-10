use inf1_svc_generic::{
    accounts::state::State,
    errs::GenSvcErr,
    instructions::interface::{
        lst_to_sol::LST_TO_SOL_IX_DISCM, sol_to_lst::SOL_TO_LST_IX_DISCM, to_retdata, IxAccs,
        IxPreAccs, IxPreAccsDestr, IxSufAccs, IxSufAccsDestr,
    },
};
use inf1_svc_generic_program::instructions::interface::{
    IxData, IX_IS_SIGNER, IX_IS_WRITER, IX_PRE_ACCS_LEN, IX_SUF_ACCS_IDX_POOL_PROG,
    IX_SUF_ACCS_IDX_POOL_PROGDATA, IX_SUF_ACCS_IDX_STATE,
};
use inf1_svc_generic_program_test::{CONST_KEYS_OWNED, CONST_PDAS};
use inf1_test_utils::{
    assert_jiminy_prog_err, keys_signer_writable_to_metas, mock_gen_svc_state, mock_mint,
    mock_progdata_acc, mock_sys_acc, mollusk_exec, perturb_ix_key_flat_map_gen, raw_mint,
    silence_mollusk_logs, u64_strat, AccountMap,
};
use jiminy_entrypoint::program_error::{ProgramError, INVALID_ARGUMENT};
use proptest::prelude::*;
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

use crate::common::SVM;

const LST_MINT: [u8; 32] = [3u8; 32];

type IxKeys = IxAccs<IxPreAccs<[u8; 32]>, IxSufAccs<[u8; 32]>>;

fn interface_ix<const DISCM: u8>(keys: &IxKeys, amt: u64) -> Instruction {
    Instruction {
        program_id: Pubkey::new_from_array(*CONST_KEYS_OWNED.program()),
        accounts: keys_signer_writable_to_metas(keys.seq(), IX_IS_SIGNER.seq(), IX_IS_WRITER.seq()),
        data: IxData::<DISCM>::new(amt).as_buf().into(),
    }
}

fn interface_keys() -> IxKeys {
    IxKeys {
        pre: IxPreAccs::from_destr(IxPreAccsDestr { lst_mint: LST_MINT }),
        suf: IxSufAccs::from_destr(IxSufAccsDestr {
            state: CONST_PDAS.state().0,
            pool_state: [0u8; 32], // free
            pool_prog: *CONST_KEYS_OWNED.pool_prog(),
            pool_progdata: CONST_PDAS.pool_progdata().0,
        }),
    }
}

fn interface_test_accs(keys: &IxKeys, state_slot: u64, progdata_slot: u64) -> AccountMap {
    let state = State {
        manager: *CONST_KEYS_OWNED.init_manager(),
        last_upgrade_slot: state_slot,
    };

    let accs = IxAccs {
        pre: IxPreAccs::from_destr(IxPreAccsDestr {
            lst_mint: mock_mint(raw_mint(None, None, 0, 0)),
        }),
        suf: IxSufAccs::from_destr(IxSufAccsDestr {
            state: mock_gen_svc_state(state, Pubkey::new_from_array(*CONST_KEYS_OWNED.program())),
            pool_state: mock_sys_acc(1_000_000_000),
            pool_prog: mock_sys_acc(1_000_000_000),
            pool_progdata: mock_progdata_acc(progdata_slot),
        }),
    };

    keys.seq()
        .copied()
        .map(Into::into)
        .zip(accs.seq().cloned())
        .collect()
}

/// Executes the interface instruction. On success asserts return data matches
/// `to_retdata(&(amt..=amt))`. On error asserts the provided error.
fn interface_test(
    ix: &Instruction,
    bef: &AccountMap,
    expected_err: Option<impl Into<ProgramError>>,
) {
    let result = SVM.with(|svm| mollusk_exec(svm, core::slice::from_ref(ix), bef));

    match expected_err {
        None => {
            let ok = result.unwrap();
            // discm dont care
            let amt = IxData::<0>::parse_no_discm(ix.data.last_chunk().unwrap());
            assert_eq!(ok.return_data, to_retdata(&(amt..=amt)).to_vec());
        }
        Some(e) => {
            assert_jiminy_prog_err(&result.unwrap_err(), e);
        }
    }
}

#[test]
fn lst_to_sol_upgrade_mismatch() {
    let keys = interface_keys();
    let am = interface_test_accs(&keys, 1, 999);
    let ix = interface_ix::<LST_TO_SOL_IX_DISCM>(&keys, 100);
    interface_test(
        &ix,
        &am,
        Some(ProgramError::custom(
            GenSvcErr::UnexpectedProgramUpgrade.into(),
        )),
    );
}

#[test]
fn sol_to_lst_upgrade_mismatch() {
    let keys = interface_keys();
    let am = interface_test_accs(&keys, 1, 999);
    let ix = interface_ix::<SOL_TO_LST_IX_DISCM>(&keys, 100);
    interface_test(
        &ix,
        &am,
        Some(ProgramError::custom(
            GenSvcErr::UnexpectedProgramUpgrade.into(),
        )),
    );
}

fn correct_strat<const DISCM: u8>() -> impl Strategy<Value = (Instruction, AccountMap)> {
    (u64_strat(None), u64_strat(None)).prop_map(move |(amt, slot)| {
        let keys = interface_keys();
        let am = interface_test_accs(&keys, slot, slot);
        (interface_ix::<DISCM>(&keys, amt), am)
    })
}

fn mismatch_strat<const DISCM: u8>() -> impl Strategy<Value = (Instruction, AccountMap)> {
    (u64_strat(None), u64_strat(None))
        .prop_flat_map(|(state_slot, progdata_slot)| {
            (Just(state_slot), Just(progdata_slot))
                .prop_filter("slots must differ", |(s, p)| s != p)
        })
        .prop_map(move |(state_slot, progdata_slot)| {
            let keys = interface_keys();
            let am = interface_test_accs(&keys, state_slot, progdata_slot);
            (interface_ix::<DISCM>(&keys, 0), am)
        })
}

fn perturbed_acc_key_strat<const DISCM: u8>(
    idx: usize,
) -> impl Strategy<Value = (Instruction, AccountMap)> {
    correct_strat::<DISCM>()
        .prop_flat_map(move |(ix, am)| {
            (
                perturb_ix_key_flat_map_gen(idx)(ix.clone()),
                Just(ix),
                Just(am),
            )
        })
        .prop_map(move |(new_ix, old_ix, mut am)| {
            let [old_pk, new_pk] = [&old_ix, &new_ix].map(|ix| ix.accounts[idx].pubkey);
            let acc = am.remove(&old_pk).unwrap();
            am.insert(new_pk, acc);
            (new_ix, am)
        })
}

proptest! {
    #[test]
    fn lst_to_sol_correct_pt(
        (ix, am) in correct_strat::<LST_TO_SOL_IX_DISCM>(),
    ) {
        silence_mollusk_logs();
        interface_test(&ix, &am,  Option::<ProgramError>::None);
    }

    #[test]
    fn lst_to_sol_upgrade_mismatch_pt(
        (ix, am) in mismatch_strat::<LST_TO_SOL_IX_DISCM>(),
    ) {
        silence_mollusk_logs();
        interface_test(
            &ix,
            &am,
            Some(ProgramError::custom(GenSvcErr::UnexpectedProgramUpgrade.into()))
        );
    }

    #[test]
    fn lst_to_sol_wrong_state_pt(
        (ix, am) in perturbed_acc_key_strat::<LST_TO_SOL_IX_DISCM>(
            IX_PRE_ACCS_LEN + IX_SUF_ACCS_IDX_STATE
        )
    ) {
        silence_mollusk_logs();
        interface_test(
            &ix,
            &am,
            Some(INVALID_ARGUMENT),
        );
    }

    #[test]
    fn lst_to_sol_wrong_pool_prog_pt(
        (ix, am) in perturbed_acc_key_strat::<LST_TO_SOL_IX_DISCM>(
            IX_PRE_ACCS_LEN + IX_SUF_ACCS_IDX_POOL_PROG
        )
    ) {
        silence_mollusk_logs();
        interface_test(
            &ix,
            &am,
            Some(INVALID_ARGUMENT),
        );
    }

    #[test]
    fn lst_to_sol_wrong_pool_progdata_pt(
        (ix, am) in perturbed_acc_key_strat::<LST_TO_SOL_IX_DISCM>(
            IX_PRE_ACCS_LEN + IX_SUF_ACCS_IDX_POOL_PROGDATA
        )
    ) {
        silence_mollusk_logs();
        interface_test(
            &ix,
            &am,
            Some(INVALID_ARGUMENT),
        );
    }

    #[test]
    fn sol_to_lst_correct_pt(
        (ix, am) in correct_strat::<SOL_TO_LST_IX_DISCM>(),
    ) {
        silence_mollusk_logs();
        interface_test(&ix, &am, Option::<ProgramError>::None);
    }

    #[test]
    fn sol_to_lst_upgrade_mismatch_pt(
        (ix, am) in mismatch_strat::<SOL_TO_LST_IX_DISCM>(),
    ) {
        silence_mollusk_logs();
        interface_test(
            &ix,
            &am,
            Some(ProgramError::custom(GenSvcErr::UnexpectedProgramUpgrade.into()))
        );
    }

    #[test]
    fn sol_to_lst_wrong_state_pt(
        (ix, am) in perturbed_acc_key_strat::<SOL_TO_LST_IX_DISCM>(
            IX_PRE_ACCS_LEN + IX_SUF_ACCS_IDX_STATE
        )
    ) {
        silence_mollusk_logs();
        interface_test(
            &ix,
            &am,
            Some(INVALID_ARGUMENT),
        );
    }

    #[test]
    fn sol_to_lst_wrong_pool_prog_pt(
        (ix, am) in perturbed_acc_key_strat::<SOL_TO_LST_IX_DISCM>(
            IX_PRE_ACCS_LEN + IX_SUF_ACCS_IDX_POOL_PROG
        )
    ) {
        silence_mollusk_logs();
        interface_test(
            &ix,
            &am,
            Some(INVALID_ARGUMENT),
        );
    }

    #[test]
    fn sol_to_lst_wrong_pool_progdata_pt(
        (ix, am) in perturbed_acc_key_strat::<SOL_TO_LST_IX_DISCM>(
            IX_PRE_ACCS_LEN + IX_SUF_ACCS_IDX_POOL_PROGDATA
        )
    ) {
        silence_mollusk_logs();
        interface_test(
            &ix,
            &am,
            Some(INVALID_ARGUMENT),
        );
    }
}
