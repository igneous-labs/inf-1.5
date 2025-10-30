use inf1_ctl_jiminy::{
    accounts::{lst_state_list::LstStatePackedList, pool_state::PoolStatePacked},
    instructions::admin::add_lst::{
        AddLstIxData, AddLstIxKeysOwned, NewAddLstIxAccsBuilder, ADD_LST_IX_IS_SIGNER,
        ADD_LST_IX_IS_WRITER,
    },
    keys::{
        ATOKEN_ID, LST_STATE_LIST_ID, POOL_STATE_ID, PROTOCOL_FEE_ID, SYS_PROG_ID, TOKENKEG_ID,
    },
    typedefs::lst_state::LstState,
    ID,
};
use inf1_svc_ag_core::inf1_svc_spl_core::keys::spl::ID as SPL_SVC;
use inf1_test_utils::{
    acc_bef_aft, assert_diffs_lst_state_list, find_pool_reserves_ata,
    find_protocol_fee_accumulator_ata, fixtures_accounts_opt_cloned, keys_signer_writable_to_metas,
    upsert_account, LstStateListChanges, PkAccountTup, ALL_FIXTURES, JITOSOL_MINT,
};

use mollusk_svm::result::{InstructionResult, ProgramResult};

use solana_account::Account;
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

use crate::common::SVM;

fn add_lst_ix_keys_owned(
    admin: &[u8; 32],
    payer: &[u8; 32],
    mint: &[u8; 32],
    token_program: &[u8; 32],
    sol_value_calculator: &[u8; 32],
) -> AddLstIxKeysOwned {
    let (pool_reserves, _) = find_pool_reserves_ata(token_program, &mint);
    let (protocol_fee_accumulator, _) = find_protocol_fee_accumulator_ata(token_program, &mint);

    NewAddLstIxAccsBuilder::start()
        .with_admin(*admin)
        .with_payer(*payer)
        .with_lst_mint(*mint)
        .with_pool_reserves(pool_reserves.to_bytes())
        .with_protocol_fee_accumulator(protocol_fee_accumulator.to_bytes())
        .with_protocol_fee_accumulator_auth(PROTOCOL_FEE_ID)
        .with_sol_value_calculator(*sol_value_calculator)
        .with_pool_state(POOL_STATE_ID)
        .with_lst_state_list(LST_STATE_LIST_ID)
        .with_associated_token_program(ATOKEN_ID)
        .with_system_program(SYS_PROG_ID)
        .with_lst_token_program(*token_program)
        .build()
}

fn add_lst_ix(keys: &AddLstIxKeysOwned) -> Instruction {
    let accounts = keys_signer_writable_to_metas(
        keys.0.iter(),
        ADD_LST_IX_IS_SIGNER.0.iter(),
        ADD_LST_IX_IS_WRITER.0.iter(),
    );
    Instruction {
        program_id: Pubkey::new_from_array(ID),
        accounts,
        data: AddLstIxData::as_buf().into(),
    }
}

fn add_lst_fixtures_accounts_opt(keys: &AddLstIxKeysOwned) -> Vec<PkAccountTup> {
    fixtures_accounts_opt_cloned(keys.0.iter().copied()).collect()
}

fn assert_correct_add(
    bef: &[PkAccountTup],
    aft: &[PkAccountTup],
    mint: &[u8; 32],
    token_program: &[u8; 32],
    expected_sol_value_calculator: &[u8; 32],
) {
    let (_, pool_reserves_bump) = find_pool_reserves_ata(token_program, mint);
    let (_, protocol_fee_accumulator_bump) = find_protocol_fee_accumulator_ata(token_program, mint);

    let lst_state_lists = acc_bef_aft(&Pubkey::new_from_array(LST_STATE_LIST_ID), bef, aft);
    let [lst_state_list_bef, lst_state_list_aft]: [Vec<_>; 2] =
        lst_state_lists.each_ref().map(|a| {
            LstStatePackedList::of_acc_data(&a.data)
                .unwrap()
                .0
                .iter()
                .map(|x| x.into_lst_state())
                .collect()
        });

    let diffs = LstStateListChanges::new(&lst_state_list_bef)
        .with_push(LstState {
            is_input_disabled: 0,
            pool_reserves_bump,
            protocol_fee_accumulator_bump,
            padding: [0u8; 5],
            sol_value: 0,
            mint: *mint,
            sol_value_calculator: *expected_sol_value_calculator,
        })
        .build();

    assert_diffs_lst_state_list(&diffs, &lst_state_list_bef, &lst_state_list_aft);
}

#[test]
fn add_lst_jitosol_fixture() {
    let pool_pk = Pubkey::new_from_array(POOL_STATE_ID);
    let pool_acc = ALL_FIXTURES
        .get(&pool_pk)
        .expect("missing pool state fixture");
    let pool = PoolStatePacked::of_acc_data(&pool_acc.data)
        .unwrap()
        .into_pool_state();

    let admin = pool.admin;
    let token_program = &TOKENKEG_ID;
    let sol_value_calculator = &SPL_SVC;

    let keys = add_lst_ix_keys_owned(
        &admin,
        &admin,
        &JITOSOL_MINT.to_bytes(),
        token_program,
        sol_value_calculator,
    );

    let ix = add_lst_ix(&keys);
    let mut accounts = add_lst_fixtures_accounts_opt(&keys);

    upsert_account(
        &mut accounts,
        (
            Pubkey::new_from_array(admin),
            Account {
                lamports: u64::MAX,
                ..Default::default()
            },
        ),
    );

    upsert_account(
        &mut accounts,
        (
            Pubkey::new_from_array(PROTOCOL_FEE_ID),
            Account {
                ..Default::default()
            },
        ),
    );

    let InstructionResult {
        program_result,
        resulting_accounts,
        ..
    } = SVM.with(|svm| svm.process_instruction(&ix, &accounts));

    assert_eq!(program_result, ProgramResult::Success);

    assert_correct_add(
        &accounts,
        &resulting_accounts,
        &JITOSOL_MINT.to_bytes(),
        token_program,
        sol_value_calculator,
    );
}
