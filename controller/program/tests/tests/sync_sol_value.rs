use inf1_core::instructions::sync_sol_value::{
    sync_sol_value_ix_is_signer, sync_sol_value_ix_is_writer, sync_sol_value_ix_keys_owned,
    SyncSolValueIxAccs,
};
use inf1_ctl_jiminy::{
    instructions::sync_sol_value::{
        NewSyncSolValueIxPreAccsBuilder, SyncSolValueIxData, SyncSolValueIxPreKeysOwned,
    },
    keys::{LST_STATE_LIST_ID, POOL_STATE_ID},
    ID,
};
use inf1_svc_ag_core::{
    inf1_svc_lido_core::solido_legacy_core::TOKENKEG_PROGRAM, instructions::SvcCalcAccsAg, SvcAgTy,
};
use inf1_test_utils::{keys_signer_writable_to_metas, PkAccountTup, ALL_FIXTURES, JUPSOL_MINT};
use mollusk_svm::result::{InstructionResult, ProgramResult};
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

use crate::common::{find_pool_reserves, jupsol_fixtures_svc_suf, JUPSOL_FIXTURE_LST_IDX, SVM};

type SyncSolValueKeysBuilder =
    SyncSolValueIxAccs<[u8; 32], SyncSolValueIxPreKeysOwned, SvcCalcAccsAg>;

fn sync_sol_value_ix_pre_keys_owned(
    token_program: &[u8; 32],
    mint: [u8; 32],
) -> SyncSolValueIxPreKeysOwned {
    NewSyncSolValueIxPreAccsBuilder::start()
        .with_lst_mint(mint)
        .with_lst_state_list(LST_STATE_LIST_ID)
        .with_pool_state(POOL_STATE_ID)
        .with_pool_reserves(find_pool_reserves(token_program, &mint).0.to_bytes())
        .build()
}

fn sync_sol_value_ix(builder: &SyncSolValueKeysBuilder, lst_idx: u32) -> Instruction {
    let accounts = keys_signer_writable_to_metas(
        sync_sol_value_ix_keys_owned(builder).seq(),
        sync_sol_value_ix_is_signer(builder).seq(),
        sync_sol_value_ix_is_writer(builder).seq(),
    );
    Instruction {
        program_id: Pubkey::new_from_array(ID),
        accounts,
        data: SyncSolValueIxData::new(lst_idx).as_buf().into(),
    }
}

fn sync_sol_value_fixtures_accounts(builder: &SyncSolValueKeysBuilder) -> Vec<PkAccountTup> {
    sync_sol_value_ix_keys_owned(builder)
        .seq()
        .map(|pk| {
            let pk = Pubkey::new_from_array(*pk);
            let (k, v) = ALL_FIXTURES.get_key_value(&pk).unwrap();
            (*k, v.clone())
        })
        .collect()
}

#[test]
fn sync_sol_value_jupsol_fixture() {
    let ix_prefix = sync_sol_value_ix_pre_keys_owned(&TOKENKEG_PROGRAM, JUPSOL_MINT.to_bytes());
    let builder = SyncSolValueKeysBuilder {
        ix_prefix,
        calc_prog: *SvcAgTy::SanctumSplMulti(()).svc_program_id(),
        calc: jupsol_fixtures_svc_suf(),
    };
    let ix = sync_sol_value_ix(&builder, JUPSOL_FIXTURE_LST_IDX as u32);
    let accounts = sync_sol_value_fixtures_accounts(&builder);

    let InstructionResult { program_result, .. } =
        SVM.with(|svm| svm.process_instruction(&ix, &accounts));

    assert!(
        matches!(program_result, ProgramResult::Success),
        "{program_result:#?}"
    );
}
