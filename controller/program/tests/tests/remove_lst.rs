use inf1_ctl_jiminy::{
    accounts::{
        lst_state_list::{LstStatePackedList, LstStatePackedListMut},
        pool_state::PoolStatePacked,
    },
    instructions::admin::remove_lst::{
        NewRemoveLstIxAccsBuilder, RemoveLstIxData, RemoveLstIxKeysOwned, REMOVE_LST_IX_IS_SIGNER,
        REMOVE_LST_IX_IS_WRITER,
    },
    keys::{LST_STATE_LIST_ID, POOL_STATE_ID, PROTOCOL_FEE_ID, TOKENKEG_ID},
    ID,
};
use inf1_test_utils::{
    acc_bef_aft, assert_diffs_lst_state_list, find_pool_reserves_ata,
    find_protocol_fee_accumulator_ata, fixtures_accounts_opt_cloned, keys_signer_writable_to_metas,
    lst_state_list_account, mock_token_acc, raw_token_acc, upsert_account, LstStateListChanges,
    PkAccountTup, ALL_FIXTURES, JUPSOL_MINT,
};

use mollusk_svm::result::{InstructionResult, ProgramResult};
use solana_account::Account;
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

use crate::common::SVM;

fn remove_lst_ix_keys_owned(
    admin: &[u8; 32],
    refund_rent_to: &[u8; 32],
    mint: &[u8; 32],
    token_program: &[u8; 32],
) -> RemoveLstIxKeysOwned {
    let (pool_reserves, _) = find_pool_reserves_ata(token_program, mint);
    let (protocol_fee_accumulator, _) = find_protocol_fee_accumulator_ata(token_program, mint);

    NewRemoveLstIxAccsBuilder::start()
        .with_admin(*admin)
        .with_refund_rent_to(*refund_rent_to)
        .with_lst_mint(*mint)
        .with_pool_reserves(pool_reserves.to_bytes())
        .with_protocol_fee_accumulator(protocol_fee_accumulator.to_bytes())
        .with_protocol_fee_accumulator_auth(PROTOCOL_FEE_ID)
        .with_pool_state(POOL_STATE_ID)
        .with_lst_state_list(LST_STATE_LIST_ID)
        .with_lst_token_program(*token_program)
        .build()
}

fn remove_lst_ix(keys: &RemoveLstIxKeysOwned, lst_idx: u32) -> Instruction {
    let accounts = keys_signer_writable_to_metas(
        keys.0.iter(),
        REMOVE_LST_IX_IS_SIGNER.0.iter(),
        REMOVE_LST_IX_IS_WRITER.0.iter(),
    );
    Instruction {
        program_id: Pubkey::new_from_array(ID),
        accounts,
        data: RemoveLstIxData::new(lst_idx).as_buf().into(),
    }
}

fn remove_lst_fixtures_accounts_opt(keys: &RemoveLstIxKeysOwned) -> Vec<PkAccountTup> {
    fixtures_accounts_opt_cloned(keys.0.iter().copied()).collect()
}

fn assert_correct_remove(bef: &[PkAccountTup], aft: &[PkAccountTup], mint: &[u8; 32]) {
    let lst_state_lists = acc_bef_aft(&Pubkey::new_from_array(LST_STATE_LIST_ID), bef, aft);
    let [_, lst_state_list_acc_aft] = lst_state_lists;

    let [lst_state_list_bef, lst_state_list_aft]: [Vec<_>; 2] =
        lst_state_lists.each_ref().map(|a| {
            LstStatePackedList::of_acc_data(&a.data)
                .unwrap()
                .0
                .iter()
                .map(|x| x.into_lst_state())
                .collect()
        });

    let bef_len = lst_state_list_bef.len();

    if bef_len == 1 {
        assert!(lst_state_list_acc_aft.data.is_empty() && lst_state_list_acc_aft.lamports == 0);
    } else {
        let diffs = LstStateListChanges::new(&lst_state_list_bef)
            .with_del_by_mint(mint)
            .build();

        assert_diffs_lst_state_list(&diffs, &lst_state_list_bef, &lst_state_list_aft);
    }
}

#[test]
fn remove_lst_jupsol_fixture() {
    let pool_pk = Pubkey::new_from_array(POOL_STATE_ID);
    let pool_acc = ALL_FIXTURES
        .get(&pool_pk)
        .expect("missing pool state fixture");
    let pool = PoolStatePacked::of_acc_data(&pool_acc.data)
        .unwrap()
        .into_pool_state();

    let admin = pool.admin;
    let token_program = &TOKENKEG_ID;

    // Find jupSOL in the list to get its index
    let lst_state_list_acc = ALL_FIXTURES
        .get(&Pubkey::new_from_array(LST_STATE_LIST_ID))
        .expect("missing lst state list fixture");
    let lst_state_list = LstStatePackedList::of_acc_data(&lst_state_list_acc.data)
        .unwrap()
        .0;

    let jupsol_idx = lst_state_list
        .iter()
        .position(|packed| {
            let lst = unsafe { packed.as_lst_state() };
            lst.mint == JUPSOL_MINT.to_bytes()
        })
        .expect("jupSOL not found in fixture list");

    let mut lst_state_list_data = lst_state_list_acc.data.clone();
    let lst_state_list = LstStatePackedListMut::of_acc_data(&mut lst_state_list_data).unwrap();
    let lst_state = lst_state_list.0.get_mut(jupsol_idx).unwrap();
    // safety: account data is 8-byte aligned
    unsafe { lst_state.as_lst_state_mut().sol_value = 0 };

    let keys = remove_lst_ix_keys_owned(
        &admin,
        &admin, // refund to admin
        &JUPSOL_MINT.to_bytes(),
        token_program,
    );

    let ix = remove_lst_ix(&keys, jupsol_idx as u32);
    let mut accounts = remove_lst_fixtures_accounts_opt(&keys);

    // Upsert the modified LST state list
    upsert_account(
        &mut accounts,
        (
            Pubkey::new_from_array(LST_STATE_LIST_ID),
            lst_state_list_account(lst_state_list_data),
        ),
    );

    upsert_account(
        &mut accounts,
        (
            Pubkey::new_from_array(admin),
            Account {
                lamports: u32::MAX as u64, // avoid overflow
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

    // Add the ATAs with zero balance
    let (pool_reserves_addr, _) = find_pool_reserves_ata(token_program, &JUPSOL_MINT.to_bytes());
    let (protocol_fee_accumulator_addr, _) =
        find_protocol_fee_accumulator_ata(token_program, &JUPSOL_MINT.to_bytes());

    upsert_account(
        &mut accounts,
        (
            pool_reserves_addr,
            mock_token_acc(raw_token_acc(JUPSOL_MINT.to_bytes(), POOL_STATE_ID, 0)),
        ),
    );

    upsert_account(
        &mut accounts,
        (
            protocol_fee_accumulator_addr,
            mock_token_acc(raw_token_acc(JUPSOL_MINT.to_bytes(), PROTOCOL_FEE_ID, 0)),
        ),
    );

    let InstructionResult {
        program_result,
        resulting_accounts,
        ..
    } = SVM.with(|svm| svm.process_instruction(&ix, &accounts));

    assert_eq!(program_result, ProgramResult::Success);

    assert_correct_remove(&accounts, &resulting_accounts, &JUPSOL_MINT.to_bytes());
}
