use generic_array_struct::generic_array_struct;
use inf1_ctl_jiminy::{
    accounts::pool_state::PoolState,
    err::Inf1CtlErr,
    instructions::protocol_fee::withdraw_protocol_fees::{
        NewWithdrawProtocolFeesIxAccsBuilder, WithdrawProtocolFeesIxAccsBuilder,
        WithdrawProtocolFeesIxData, WithdrawProtocolFeesIxKeysOwned,
        WITHDRAW_PROTOCOL_FEES_IX_ACCS_IDX_BENEFICIARY,
        WITHDRAW_PROTOCOL_FEES_IX_ACCS_IDX_PROTOCOL_FEE_ACCUMULATOR,
        WITHDRAW_PROTOCOL_FEES_IX_ACCS_IDX_WITHDRAW_TO, WITHDRAW_PROTOCOL_FEES_IX_IS_SIGNER,
        WITHDRAW_PROTOCOL_FEES_IX_IS_WRITER,
    },
    keys::{POOL_STATE_ID, PROTOCOL_FEE_ID},
    program_err::Inf1CtlCustomProgErr,
};
use inf1_svc_ag_core::inf1_svc_lido_core::solido_legacy_core::TOKENKEG_PROGRAM;
use inf1_test_utils::{
    acc_bef_aft, any_normal_pk, any_pool_state, assert_jiminy_prog_err, assert_token_acc_diffs,
    bals_from_supply, find_protocol_fee_accumulator_ata, gen_pool_state,
    keys_signer_writable_to_metas, mock_mint_with_prog, mock_sys_acc, mock_token_acc_with_prog,
    mollusk_exec, n_distinct_normal_pks, pool_state_account, raw_mint, raw_token_acc,
    silence_mollusk_logs, token_acc_bal_diff_changed, AccountMap, AnyPoolStateArgs,
    GenPoolStateArgs, PoolStateBools, PoolStatePks, ALL_FIXTURES,
};
use jiminy_cpi::program_error::{
    ProgramError, ILLEGAL_OWNER, INVALID_ARGUMENT, MISSING_REQUIRED_SIGNATURE,
};
use mollusk_svm::result::{InstructionResult, ProgramResult};
use proptest::prelude::*;
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

#[derive(Debug, Clone, Copy)]
struct MintParams {
    pub supply: u64,
    pub decimals: u8,
}

#[generic_array_struct(builder pub)]
#[derive(Debug, Clone, Copy)]
struct TokenBals<T> {
    pub accum: T,
    pub withdraw_to: T,
}

type TokenBalsU64 = TokenBals<u64>;

impl TokenBalsU64 {
    #[inline]
    pub const fn zeros() -> Self {
        Self([0; TOKEN_BALS_LEN])
    }
}

fn withdraw_protocol_fees_test_accs(
    keys: &WithdrawProtocolFeesIxKeysOwned,
    pool: PoolState,
    MintParams { supply, decimals }: MintParams,
    bals: TokenBalsU64,
) -> AccountMap {
    // dont care abt lamports of sys accounts, shouldnt affect anything
    const LAMPORTS: u64 = 1_000_000_000;

    let token_accs = TokenBals(bals.0.map(|amt| pf_owned_token_acc(*keys.lst_mint(), amt)));
    let token_prog = *keys.token_program();

    let accs = NewWithdrawProtocolFeesIxAccsBuilder::start()
        .with_beneficiary(mock_sys_acc(LAMPORTS))
        .with_lst_mint(mock_mint_with_prog(gen_mint(supply, decimals), token_prog))
        .with_protocol_fee_accumulator_auth(mock_sys_acc(0))
        .with_token_program(ALL_FIXTURES.get(&TOKENKEG_PROGRAM.into()).unwrap().clone())
        .with_protocol_fee_accumulator(mock_token_acc_with_prog(*token_accs.accum(), token_prog))
        .with_withdraw_to(mock_token_acc_with_prog(
            *token_accs.withdraw_to(),
            token_prog,
        ))
        .with_pool_state(pool_state_account(pool))
        .build();
    keys.0.into_iter().map(Into::into).zip(accs.0).collect()
}

/// Returns amt ix arg
fn withdraw_protocol_fees_test(
    ix: &Instruction,
    bef: &AccountMap,
    expected_err: Option<impl Into<ProgramError>>,
) -> u64 {
    let (
        bef,
        InstructionResult {
            program_result,
            resulting_accounts,
            ..
        },
    ) = SVM.with(|svm| mollusk_exec(svm, ix, bef));
    let aft: AccountMap = resulting_accounts.into_iter().collect();

    let amt_data: &[u8; 8] = &ix.data[1..].try_into().unwrap();
    let amt_u64 = WithdrawProtocolFeesIxData::parse_no_discm(amt_data);
    let amt: i128 = amt_u64.into();
    let [pf, wt] = [
        WITHDRAW_PROTOCOL_FEES_IX_ACCS_IDX_PROTOCOL_FEE_ACCUMULATOR,
        WITHDRAW_PROTOCOL_FEES_IX_ACCS_IDX_WITHDRAW_TO,
    ]
    .map(|i| {
        let pk = &ix.accounts[i].pubkey;
        acc_bef_aft(pk, &bef, &aft).map(|a| RawTokenAccount::of_acc_data(&a.data).unwrap())
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

fn kb_with_const_pdas<const A: bool, const B: bool, const C: bool, const D: bool, const E: bool>(
    b: WithdrawProtocolFeesIxAccsBuilder<[u8; 32], A, B, C, false, D, false, E>,
) -> WithdrawProtocolFeesIxAccsBuilder<[u8; 32], A, B, C, true, D, true, E> {
    b.with_pool_state(POOL_STATE_ID)
        .with_protocol_fee_accumulator_auth(PROTOCOL_FEE_ID)
}

fn kb_with_mint<const A: bool, const B: bool, const C: bool, const D: bool>(
    mint: [u8; 32],
    token_prog: [u8; 32],
    b: WithdrawProtocolFeesIxAccsBuilder<[u8; 32], A, B, false, C, false, D, false>,
) -> WithdrawProtocolFeesIxAccsBuilder<[u8; 32], A, B, true, C, true, D, true> {
    b.with_lst_mint(mint)
        .with_token_program(token_prog)
        .with_protocol_fee_accumulator(
            find_protocol_fee_accumulator_ata(&token_prog, &mint)
                .0
                .to_bytes(),
        )
}

fn kb_with_tokenkeg_mint<const A: bool, const B: bool, const C: bool, const D: bool>(
    mint: [u8; 32],
    b: WithdrawProtocolFeesIxAccsBuilder<[u8; 32], A, B, false, C, false, D, false>,
) -> WithdrawProtocolFeesIxAccsBuilder<[u8; 32], A, B, true, C, true, D, true> {
    kb_with_mint(mint, TOKENKEG_PROGRAM, b)
}

fn kb_tokenkeg_mint(
    mint: [u8; 32],
) -> WithdrawProtocolFeesIxAccsBuilder<[u8; 32], false, false, true, true, true, true, true> {
    kb_with_tokenkeg_mint(
        mint,
        kb_with_const_pdas(NewWithdrawProtocolFeesIxAccsBuilder::start()),
    )
}

#[test]
fn withdraw_protocol_fees_test_correct_basic() {
    const BALS: TokenBalsU64 = TokenBalsU64::zeros()
        .const_with_accum(1_000_000_000_000)
        .const_with_withdraw_to(50);
    const MINT: MintParams = MintParams {
        supply: 10_000_000_000_000,
        decimals: 9,
    };
    const AMT: u64 = 500_000_000;

    // 69 + to avoid colliding with system prog
    let [ben, mint, wt] = core::array::from_fn(|i| [69 + u8::try_from(i).unwrap(); 32]);
    let pool = gen_pool_state(GenPoolStateArgs {
        pks: PoolStatePks::default().with_protocol_fee_beneficiary(ben),
        version: 1,
        ..Default::default()
    });
    let keys = kb_tokenkeg_mint(mint)
        .with_beneficiary(ben)
        .with_withdraw_to(wt)
        .build();
    let ret = withdraw_protocol_fees_test(
        &withdraw_protocol_fees_ix(&keys, AMT),
        &withdraw_protocol_fees_test_accs(&keys, pool, MINT, BALS),
        Option::<ProgramError>::None,
    );

    assert_eq!(ret, AMT);
}

fn to_inp(
    (keys, pool, amt, bals, mint): (
        WithdrawProtocolFeesIxKeysOwned,
        PoolState,
        u64,
        TokenBalsU64,
        MintParams,
    ),
) -> (Instruction, AccountMap) {
    (
        withdraw_protocol_fees_ix(&keys, amt),
        withdraw_protocol_fees_test_accs(&keys, pool, mint, bals),
    )
}

fn valid_bals_and_supply_strat(supply: u64) -> impl Strategy<Value = (TokenBalsU64, u64)> {
    bals_from_supply(supply).prop_map(|([accum, withdraw_to], supply)| {
        (
            NewTokenBalsBuilder::start()
                .with_accum(accum)
                .with_withdraw_to(withdraw_to)
                .build(),
            supply,
        )
    })
}

fn valid_bals_and_mint_strat() -> impl Strategy<Value = (TokenBalsU64, MintParams)> {
    (
        any::<u8>(),
        (0..=u64::MAX).prop_flat_map(valid_bals_and_supply_strat),
    )
        .prop_map(|(decimals, (bals, supply))| (bals, MintParams { decimals, supply }))
}

fn valid_amt_strat(accum_bal: u64) -> impl Strategy<Value = u64> {
    0..=accum_bal
}

fn valid_args_strat() -> impl Strategy<Value = (u64, TokenBalsU64, MintParams)> {
    valid_bals_and_mint_strat()
        .prop_flat_map(|(b, m)| (valid_amt_strat(*b.accum()), Just(b), Just(m)))
}

fn correct_strat() -> impl Strategy<Value = (Instruction, AccountMap)> {
    (
        n_distinct_normal_pks(),
        any_pool_state(AnyPoolStateArgs {
            bools: PoolStateBools::normal(),
            ..Default::default()
        }),
        valid_args_strat(),
    )
        .prop_map(|([wt_pk, mint_pk], ps, (amt, bals, mint))| {
            (
                kb_tokenkeg_mint(mint_pk)
                    .with_beneficiary(ps.protocol_fee_beneficiary)
                    .with_withdraw_to(wt_pk)
                    .build(),
                ps,
                amt,
                bals,
                mint,
            )
        })
        .prop_map(to_inp)
}

proptest! {
    #[test]
    fn withdraw_protocol_fees_correct_pt(
        (ix, bef) in correct_strat(),
    ) {
        silence_mollusk_logs();
        withdraw_protocol_fees_test(&ix, &bef, Option::<ProgramError>::None);
    }
}

fn unauthorized_strat() -> impl Strategy<Value = (Instruction, AccountMap)> {
    any_pool_state(AnyPoolStateArgs {
        bools: PoolStateBools::normal(),
        ..Default::default()
    })
    .prop_flat_map(|ps| {
        (
            any_normal_pk().prop_filter("", move |pk| *pk != ps.protocol_fee_beneficiary),
            n_distinct_normal_pks(),
            Just(ps),
            valid_args_strat(),
        )
    })
    .prop_map(|(unauth, [wt_pk, mint_pk], ps, (amt, bals, mint))| {
        (
            kb_tokenkeg_mint(mint_pk)
                .with_beneficiary(unauth)
                .with_withdraw_to(wt_pk)
                .build(),
            ps,
            amt,
            bals,
            mint,
        )
    })
    .prop_map(to_inp)
}

proptest! {
    #[test]
    fn withdraw_protocol_fees_unauthorized_pt(
        (ix, bef) in unauthorized_strat(),
    ) {
        silence_mollusk_logs();
        withdraw_protocol_fees_test(&ix, &bef, Some(INVALID_ARGUMENT));
    }
}

fn missing_sig_strat() -> impl Strategy<Value = (Instruction, AccountMap)> {
    correct_strat().prop_map(|(mut ix, bef)| {
        ix.accounts[WITHDRAW_PROTOCOL_FEES_IX_ACCS_IDX_BENEFICIARY].is_signer = false;
        (ix, bef)
    })
}

proptest! {
    #[test]
    fn withdraw_protocol_fees_missing_sig_pt(
        (ix, bef) in missing_sig_strat(),
    ) {
        silence_mollusk_logs();
        withdraw_protocol_fees_test(&ix, &bef, Some(MISSING_REQUIRED_SIGNATURE));
    }
}

fn exceed_amt_strat(accum_bal: u64) -> impl Strategy<Value = u64> {
    accum_bal + 1..
}

fn exceed_args_strat() -> impl Strategy<Value = (u64, TokenBalsU64, MintParams)> {
    (
        any::<u8>(),
        // -1 to avoid overflow in exceed_amt_strat
        (0..=u64::MAX - 1).prop_flat_map(valid_bals_and_supply_strat),
    )
        .prop_map(|(decimals, (bals, supply))| (bals, MintParams { decimals, supply }))
        .prop_flat_map(|(b, m)| (exceed_amt_strat(*b.accum()), Just(b), Just(m)))
}

fn exceed_strat() -> impl Strategy<Value = (Instruction, AccountMap)> {
    (
        n_distinct_normal_pks(),
        any_pool_state(AnyPoolStateArgs {
            bools: PoolStateBools::normal(),
            ..Default::default()
        }),
        exceed_args_strat(),
    )
        .prop_map(|([wt_pk, mint_pk], ps, (amt, bals, mint))| {
            (
                kb_tokenkeg_mint(mint_pk)
                    .with_beneficiary(ps.protocol_fee_beneficiary)
                    .with_withdraw_to(wt_pk)
                    .build(),
                ps,
                amt,
                bals,
                mint,
            )
        })
        .prop_map(to_inp)
}

proptest! {
    #[test]
    fn withdraw_protocol_fees_exceed_pt(
        (ix, bef) in exceed_strat(),
    ) {
        silence_mollusk_logs();
        withdraw_protocol_fees_test(
            &ix,
            &bef,
            Some(Inf1CtlCustomProgErr(Inf1CtlErr::NotEnoughFees))
        );
    }
}

fn disabled_strat() -> impl Strategy<Value = (Instruction, AccountMap)> {
    (
        n_distinct_normal_pks(),
        any_pool_state(AnyPoolStateArgs {
            bools: PoolStateBools::normal().with_is_disabled(Some(Just(true).boxed())),
            ..Default::default()
        }),
        valid_args_strat(),
    )
        .prop_map(|([wt_pk, mint_pk], ps, (amt, bals, mint))| {
            (
                kb_tokenkeg_mint(mint_pk)
                    .with_beneficiary(ps.protocol_fee_beneficiary)
                    .with_withdraw_to(wt_pk)
                    .build(),
                ps,
                amt,
                bals,
                mint,
            )
        })
        .prop_map(to_inp)
}

proptest! {
    #[test]
    fn withdraw_protocol_fees_disabled_pt(
        (ix, bef) in disabled_strat(),
    ) {
        silence_mollusk_logs();
        withdraw_protocol_fees_test(
            &ix,
            &bef,
            Some(Inf1CtlCustomProgErr(Inf1CtlErr::PoolDisabled))
        );
    }
}

fn rebalancing_strat() -> impl Strategy<Value = (Instruction, AccountMap)> {
    (
        n_distinct_normal_pks(),
        any_pool_state(AnyPoolStateArgs {
            bools: PoolStateBools::normal().with_is_rebalancing(Some(Just(true).boxed())),
            ..Default::default()
        }),
        valid_args_strat(),
    )
        .prop_map(|([wt_pk, mint_pk], ps, (amt, bals, mint))| {
            (
                kb_tokenkeg_mint(mint_pk)
                    .with_beneficiary(ps.protocol_fee_beneficiary)
                    .with_withdraw_to(wt_pk)
                    .build(),
                ps,
                amt,
                bals,
                mint,
            )
        })
        .prop_map(to_inp)
}

proptest! {
    #[test]
    fn withdraw_protocol_fees_rebalancing_pt(
        (ix, bef) in rebalancing_strat(),
    ) {
        silence_mollusk_logs();
        withdraw_protocol_fees_test(
            &ix,
            &bef,
            Some(Inf1CtlCustomProgErr(Inf1CtlErr::PoolRebalancing))
        );
    }
}

fn invalid_token_prog_strat() -> impl Strategy<Value = (Instruction, AccountMap)> {
    (
        n_distinct_normal_pks(),
        any_pool_state(AnyPoolStateArgs {
            bools: PoolStateBools::normal(),
            ..Default::default()
        }),
        valid_args_strat(),
    )
        .prop_map(
            |([wt_pk, mint_pk, bad_token_prog_pk], ps, (amt, bals, mint))| {
                (
                    kb_with_mint(
                        mint_pk,
                        bad_token_prog_pk,
                        kb_with_const_pdas(NewWithdrawProtocolFeesIxAccsBuilder::start()),
                    )
                    .with_beneficiary(ps.protocol_fee_beneficiary)
                    .with_withdraw_to(wt_pk)
                    .build(),
                    ps,
                    amt,
                    bals,
                    mint,
                )
            },
        )
        .prop_map(to_inp)
}

proptest! {
    #[test]
    fn withdraw_protocol_fees_invalid_token_prog_pt(
        (ix, bef) in invalid_token_prog_strat(),
    ) {
        silence_mollusk_logs();
        withdraw_protocol_fees_test(
            &ix,
            &bef,
            Some(ILLEGAL_OWNER)
        );
    }
}
