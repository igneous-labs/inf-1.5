use inf1_ctl_jiminy::{
    accounts::pool_state::PoolState,
    instructions::protocol_fee::withdraw_protocol_fees::{
        NewWithdrawProtocolFeesIxAccsBuilder, WithdrawProtocolFeesIxData,
        WithdrawProtocolFeesIxKeysOwned,
        WITHDRAW_PROTOCOL_FEES_IX_ACCS_IDX_PROTOCOL_FEE_ACCUMULATOR,
        WITHDRAW_PROTOCOL_FEES_IX_ACCS_IDX_WITHDRAW_TO, WITHDRAW_PROTOCOL_FEES_IX_IS_SIGNER,
        WITHDRAW_PROTOCOL_FEES_IX_IS_WRITER,
    },
    keys::{POOL_STATE_ID, PROTOCOL_FEE_ID},
};
use inf1_svc_ag_core::inf1_svc_lido_core::solido_legacy_core::TOKENKEG_PROGRAM;
use inf1_test_utils::{
    acc_bef_aft, assert_jiminy_prog_err, dedup_accounts, find_protocol_fee_accumulator_ata,
    gen_pool_state, keys_signer_writable_to_metas, mock_mint, mock_sys_acc, mock_token_acc,
    pool_state_account, raw_mint, raw_token_acc,
    token::{assert_token_acc_diffs, token_acc_bal_diff_changed},
    GenPoolStateArgs, PkAccountTup, PoolStatePks, ALL_FIXTURES,
};
use jiminy_cpi::program_error::ProgramError;
use mollusk_svm::result::{InstructionResult, ProgramResult};
use sanctum_spl_token_jiminy::sanctum_spl_token_core::state::{
    account::RawTokenAccount, mint::RawMint,
};
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

use crate::common::SVM;

fn withdraw_protocol_fees_ix(keys: &WithdrawProtocolFeesIxKeysOwned, amt: u64) -> Instruction {
    let accounts = keys_signer_writable_to_metas(
        keys.0.iter(),
        WITHDRAW_PROTOCOL_FEES_IX_IS_SIGNER.0.iter(),
        WITHDRAW_PROTOCOL_FEES_IX_IS_WRITER.0.iter(),
    );
    Instruction {
        program_id: Pubkey::new_from_array(inf1_ctl_jiminy::ID),
        accounts,
        data: WithdrawProtocolFeesIxData::new(amt).as_buf().into(),
    }
}

fn gen_mint(supply: u64, decimals: u8) -> RawMint {
    // dont care abt mint and freeze auth of the mint
    // for this ix
    raw_mint(None, None, supply, decimals)
}

fn pf_owned_token_acc(mint: [u8; 32], amt: u64) -> RawTokenAccount {
    raw_token_acc(mint, PROTOCOL_FEE_ID, amt)
}

struct TokenBals {
    pub accum: u64,
    pub withdraw_to: u64,
}

fn withdraw_protocol_fees_test_accs(
    keys: &WithdrawProtocolFeesIxKeysOwned,
    pool: PoolState,
    supply: u64,
    decimals: u8,
    TokenBals { accum, withdraw_to }: TokenBals,
) -> Vec<PkAccountTup> {
    // dont care abt lamports of sys accounts, shouldnt affect anything
    const LAMPORTS: u64 = 1_000_000_000;

    let [pf, wt] =
        [accum, withdraw_to].map(|amt| mock_token_acc(pf_owned_token_acc(*keys.lst_mint(), amt)));

    let accs = NewWithdrawProtocolFeesIxAccsBuilder::start()
        .with_beneficiary(mock_sys_acc(LAMPORTS))
        .with_lst_mint(mock_mint(gen_mint(supply, decimals)))
        .with_protocol_fee_accumulator_auth(mock_sys_acc(0))
        .with_token_program(ALL_FIXTURES.get(&TOKENKEG_PROGRAM.into()).unwrap().clone())
        .with_protocol_fee_accumulator(pf)
        .with_withdraw_to(wt)
        .with_pool_state(pool_state_account(pool))
        .build();
    let mut res = keys.0.into_iter().map(Into::into).zip(accs.0).collect();
    dedup_accounts(&mut res);
    res
}

/// Returns amt ix arg
fn withdraw_protocol_fees_test(
    ix: &Instruction,
    bef: &[PkAccountTup],
    expected_err: Option<impl Into<ProgramError>>,
) -> u64 {
    let InstructionResult {
        program_result,
        resulting_accounts: aft,
        ..
    } = SVM.with(|svm| svm.process_instruction(ix, bef));

    let amt_data: &[u8; 8] = &ix.data[1..].try_into().unwrap();
    let amt_u64 = WithdrawProtocolFeesIxData::parse_no_discm(amt_data);
    let amt: i128 = amt_u64.into();
    let [pf, wt] = [
        WITHDRAW_PROTOCOL_FEES_IX_ACCS_IDX_PROTOCOL_FEE_ACCUMULATOR,
        WITHDRAW_PROTOCOL_FEES_IX_ACCS_IDX_WITHDRAW_TO,
    ]
    .map(|i| {
        let pk = &ix.accounts[i].pubkey;
        acc_bef_aft(pk, bef, &aft).map(|a| RawTokenAccount::of_acc_data(&a.data).unwrap())
    });

    match expected_err {
        None => {
            assert_eq!(program_result, ProgramResult::Success);
            [(pf, -amt), (wt, amt)]
                .iter()
                .for_each(|([bef, aft], change)| {
                    assert_token_acc_diffs(bef, aft, &token_acc_bal_diff_changed(bef, *change));
                });
        }
        Some(e) => {
            assert_jiminy_prog_err(&program_result, e);
        }
    }

    amt_u64
}

#[test]
fn withdraw_protocol_fees_test_correct_basic() {
    const BALS: TokenBals = TokenBals {
        accum: 1_000_000_000_000,
        withdraw_to: 50,
    };
    const AMT: u64 = 500_000_000;
    const SUPPLY: u64 = 10_000_000_000_000;
    const DECIMALS: u8 = 9;

    // 69 + to avoid colliding with system prog
    let [ben, mint, wt] = core::array::from_fn(|i| [69 + u8::try_from(i).unwrap(); 32]);
    let pool = gen_pool_state(GenPoolStateArgs {
        pks: PoolStatePks::default().with_protocol_fee_beneficiary(ben),
        ..Default::default()
    });
    let keys = NewWithdrawProtocolFeesIxAccsBuilder::start()
        .with_beneficiary(ben)
        .with_lst_mint(mint)
        .with_withdraw_to(wt)
        .with_token_program(TOKENKEG_PROGRAM)
        .with_pool_state(POOL_STATE_ID)
        .with_protocol_fee_accumulator_auth(PROTOCOL_FEE_ID)
        .with_protocol_fee_accumulator(
            find_protocol_fee_accumulator_ata(&TOKENKEG_PROGRAM, &mint)
                .0
                .to_bytes(),
        )
        .build();
    let ret = withdraw_protocol_fees_test(
        &withdraw_protocol_fees_ix(&keys, AMT),
        &withdraw_protocol_fees_test_accs(&keys, pool, SUPPLY, DECIMALS, BALS),
        Option::<ProgramError>::None,
    );

    assert_eq!(ret, AMT);
}
