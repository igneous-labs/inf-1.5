use inf1_ctl_jiminy::{
    accounts::{
        lst_state_list::{LstStatePackedList, LstStatePackedListMut},
        pool_state::PoolStatePacked,
    },
    instructions::set_sol_value_calculator::{
        NewSetSolValueCalculatorIxPreAccsBuilder, SetSolValueCalculatorIxData,
        SetSolValueCalculatorIxPreKeysOwned,
    },
    keys::{LST_STATE_LIST_ID, POOL_STATE_ID},
    ID,
};

use inf1_svc_ag_core::{
    inf1_svc_lido_core::solido_legacy_core::TOKENKEG_PROGRAM, instructions::SvcCalcAccsAg, SvcAgTy,
};

use inf1_core::instructions::set_sol_value_calculator::{
    set_sol_value_calculator_ix_is_signer, set_sol_value_calculator_ix_is_writer,
    set_sol_value_calculator_ix_keys_owned, SetSolValueCalculatorIxAccs,
};

use inf1_test_utils::{
    acc_bef_aft, find_pool_reserves, fixtures_accounts_opt_cloned, keys_signer_writable_to_metas,
    lst_state_list_account, upsert_account, PkAccountTup, ALL_FIXTURES, JUPSOL_FIXTURE_LST_IDX,
    JUPSOL_MINT,
};

use mollusk_svm::result::{InstructionResult, ProgramResult};

use solana_account::Account;
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

use crate::common::{jupsol_fixtures_svc_suf, SVM};

type SetSolValueCalculatorKeysBuilder =
    SetSolValueCalculatorIxAccs<[u8; 32], SetSolValueCalculatorIxPreKeysOwned, SvcCalcAccsAg>;

fn set_sol_value_calculator_ix_pre_keys_owned(
    admin: [u8; 32],
    token_program: &[u8; 32],
    mint: [u8; 32],
) -> SetSolValueCalculatorIxPreKeysOwned {
    NewSetSolValueCalculatorIxPreAccsBuilder::start()
        .with_admin(admin)
        .with_lst_mint(mint)
        .with_pool_state(POOL_STATE_ID)
        .with_pool_reserves(find_pool_reserves(token_program, &mint).0.to_bytes())
        .with_lst_state_list(LST_STATE_LIST_ID)
        .build()
}

fn set_sol_value_calculator_ix(
    builder: &SetSolValueCalculatorKeysBuilder,
    lst_idx: u32,
) -> Instruction {
    let accounts = keys_signer_writable_to_metas(
        set_sol_value_calculator_ix_keys_owned(builder).seq(),
        set_sol_value_calculator_ix_is_signer(builder).seq(),
        set_sol_value_calculator_ix_is_writer(builder).seq(),
    );
    Instruction {
        program_id: Pubkey::new_from_array(ID),
        accounts,
        data: SetSolValueCalculatorIxData::new(lst_idx).as_buf().into(),
    }
}

fn set_sol_value_calculator_fixtures_accounts_opt(
    builder: &SetSolValueCalculatorKeysBuilder,
) -> Vec<PkAccountTup> {
    fixtures_accounts_opt_cloned(
        set_sol_value_calculator_ix_keys_owned(builder)
            .seq()
            .copied(),
    )
    .collect()
}

pub fn assert_correct_set(
    bef: &[PkAccountTup],
    aft: &[PkAccountTup],
    mint: &[u8; 32],
    expected_new_calc: &[u8; 32],
) {
    let [pools, lst_state_lists] = [POOL_STATE_ID, LST_STATE_LIST_ID]
        .map(|a| acc_bef_aft(&Pubkey::new_from_array(a), bef, aft));
    let [pool_bef, pool_aft] = pools.each_ref().map(|a| {
        PoolStatePacked::of_acc_data(&a.data)
            .unwrap()
            .into_pool_state()
    });
    let [lst_state_list_bef, lst_state_list_aft] = lst_state_lists
        .each_ref()
        .map(|a| LstStatePackedList::of_acc_data(&a.data).unwrap());
    let lst_state_i = lst_state_list_bef
        .0
        .iter()
        .position(|s| s.into_lst_state().mint == *mint)
        .unwrap();
    let [lst_state_bef, lst_state_aft] =
        [lst_state_list_bef, lst_state_list_aft].map(|l| l.0[lst_state_i].into_lst_state());

    assert_eq!(lst_state_bef.mint, *mint);
    assert_eq!(lst_state_bef.mint, lst_state_aft.mint);

    assert_eq!(&lst_state_aft.sol_value_calculator, expected_new_calc);

    let delta = i128::from(pool_aft.total_sol_value) - i128::from(pool_bef.total_sol_value);
    assert_eq!(
        delta,
        i128::from(lst_state_aft.sol_value) - i128::from(lst_state_bef.sol_value)
    );
}

#[test]
fn set_sol_value_calculator_jupsol_fixture() {
    let pool_pk = Pubkey::new_from_array(POOL_STATE_ID);
    let pool_acc = ALL_FIXTURES
        .get(&pool_pk)
        .expect("missing pool state fixture");

    let pool = PoolStatePacked::of_acc_data(&pool_acc.data)
        .unwrap()
        .into_pool_state();
    let admin = pool.admin;

    let ix_prefix = set_sol_value_calculator_ix_pre_keys_owned(
        admin,
        &TOKENKEG_PROGRAM,
        JUPSOL_MINT.to_bytes(),
    );
    let builder = SetSolValueCalculatorKeysBuilder {
        ix_prefix,
        calc_prog: *SvcAgTy::SanctumSplMulti(()).svc_program_id(),
        calc: jupsol_fixtures_svc_suf(),
    };
    let ix = set_sol_value_calculator_ix(&builder, JUPSOL_FIXTURE_LST_IDX as u32);
    let mut accounts = set_sol_value_calculator_fixtures_accounts_opt(&builder);

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

    let calc_pk = Pubkey::new_from_array(builder.calc_prog);
    let mut calc_acc = ALL_FIXTURES.get(&calc_pk).unwrap().clone();
    calc_acc.executable = true;
    upsert_account(&mut accounts, (calc_pk, calc_acc));

    let lsl_pk = Pubkey::new_from_array(LST_STATE_LIST_ID);
    let lsl_acc = ALL_FIXTURES.get(&lsl_pk).unwrap().clone();
    let mut lsl_data = lsl_acc.data.to_vec();

    let lsl_mut = LstStatePackedListMut::of_acc_data(&mut lsl_data).unwrap();
    let lst_mut = unsafe {
        lsl_mut
            .0
            .get_mut(JUPSOL_FIXTURE_LST_IDX)
            .unwrap()
            .as_lst_state_mut()
    };
    lst_mut.sol_value_calculator = Pubkey::new_unique().to_bytes();

    upsert_account(&mut accounts, (lsl_pk, lst_state_list_account(lsl_data)));

    let InstructionResult {
        program_result,
        resulting_accounts,
        ..
    } = SVM.with(|svm| svm.process_instruction(&ix, &accounts));

    assert_eq!(program_result, ProgramResult::Success);

    assert_correct_set(
        &accounts,
        &resulting_accounts,
        JUPSOL_MINT.as_array(),
        &jupsol_fixtures_svc_suf().svc_program_id(),
    );
}
