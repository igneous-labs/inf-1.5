use inf1_ctl_jiminy::accounts::pool_state::PoolStateV2;
use inf1_pp_core::{
    instructions::price::{IxAccs as PriceIxAccs, IxPreAccs},
    pair::Pair,
};
use inf1_pp_reserve_v2_core::{
    instructions::pricing::ReserveV2PpAccs,
    keys::CONST_KEYS_OWNED,
    typedefs::{FeeEntry, FeeEntryNanos, FeeEntryNanosDestr},
};
use inf1_test_utils::{
    assert_jiminy_prog_err, mock_reserve_v2_pricing_state_account, mock_token_acc, mollusk_exec,
    pool_state_v2_account, raw_token_acc, AccountMap,
};
use solana_account::Account;
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

use crate::common::SVM;

mod exact_in;
mod exact_out;

pub const LST_MINT: [u8; 32] = [7; 32];
pub const THRESHOLD_NANOS: u32 = 500_000_000;
pub const LAMPORTS_PER_SOL: u64 = 1_000_000_000;
pub const POOL_SOL_VALUE: u64 = 10 * LAMPORTS_PER_SOL;
pub const WSOL_BALANCE: u64 = 7 * LAMPORTS_PER_SOL;

pub type PriceIxKeysOwned = PriceIxAccs<[u8; 32], ReserveV2PpAccs>;

pub fn price_keys_owned(Pair { inp, out }: Pair<[u8; 32]>) -> PriceIxKeysOwned {
    PriceIxKeysOwned::new(IxPreAccs::new([inp, out]), ReserveV2PpAccs::MAINNET)
}

pub fn price_ix_accounts(
    keys: &PriceIxKeysOwned,
    pricing_state: Account,
    pool_state: Account,
    wsol_reserves: Account,
) -> AccountMap {
    AccountMap::from([
        (
            Pubkey::new_from_array(*keys.ix_prefix.input_mint()),
            Account::default(),
        ),
        (
            Pubkey::new_from_array(*keys.ix_prefix.output_mint()),
            Account::default(),
        ),
        (
            Pubkey::new_from_array(*keys.suf.0.pricing_state()),
            pricing_state,
        ),
        (Pubkey::new_from_array(*keys.suf.0.pool_state()), pool_state),
        (
            Pubkey::new_from_array(*keys.suf.0.wsol_reserves()),
            wsol_reserves,
        ),
    ])
}

pub fn fee_entry(
    mint: [u8; 32],
    threshold_nanos: u32,
    base_fee: u32,
    threshold_fee: u32,
    max_fee: u32,
    output_fee: u32,
) -> FeeEntry {
    FeeEntry {
        mint,
        threshold_nanos,
        fee_nanos: FeeEntryNanos::from_destr(FeeEntryNanosDestr {
            base_fee,
            threshold_fee,
            max_fee,
            output_fee,
        }),
    }
}

pub fn wsol_entry() -> FeeEntry {
    // 0% fees
    fee_entry(*CONST_KEYS_OWNED.wsol_mint(), THRESHOLD_NANOS, 0, 0, 0, 0)
}

pub fn lp_entry() -> FeeEntry {
    // 100% input fee, 0% output fee
    fee_entry(
        *CONST_KEYS_OWNED.lp_mint(),
        THRESHOLD_NANOS,
        1_000_000_000,
        1_000_000_000,
        1_000_000_000,
        0,
    )
}

pub fn lst_entry() -> FeeEntry {
    fee_entry(
        LST_MINT,
        THRESHOLD_NANOS,
        100_000_000,
        200_000_000,
        300_000_000,
        1_000_000_000, // 100% output fee
    )
}

pub fn pricing_state_account(mut entries: Vec<FeeEntry>) -> Account {
    entries.sort_unstable_by_key(|entry| entry.mint);
    mock_reserve_v2_pricing_state_account([1; 32], &entries)
}

pub fn pool_state_account(total_sol_value: u64) -> Account {
    let mut pool = PoolStateV2::init(0, *CONST_KEYS_OWNED.lp_mint());
    pool.total_sol_value = total_sol_value;
    pool_state_v2_account(pool)
}

pub fn wsol_reserves_account(amount: u64) -> Account {
    mock_token_acc(raw_token_acc(
        *CONST_KEYS_OWNED.wsol_mint(),
        *ReserveV2PpAccs::MAINNET.0.pool_state(),
        amount,
    ))
}

pub fn valid_accounts(keys: &PriceIxKeysOwned, entries: Vec<FeeEntry>) -> AccountMap {
    price_ix_accounts(
        keys,
        pricing_state_account(entries),
        pool_state_account(POOL_SOL_VALUE),
        wsol_reserves_account(WSOL_BALANCE),
    )
}

pub fn execute_success(ix: Instruction, accounts: &AccountMap, expected: u64) {
    let ok = SVM
        .with(|mollusk| mollusk_exec(mollusk, &[ix], accounts))
        .unwrap();
    assert_eq!(
        u64::from_le_bytes(ok.return_data.try_into().unwrap()),
        expected
    );
}

pub fn execute_failure(
    ix: Instruction,
    accounts: &AccountMap,
    expected: impl Into<jiminy_entrypoint::program_error::ProgramError>,
) {
    let err = SVM
        .with(|mollusk| mollusk_exec(mollusk, &[ix], accounts))
        .unwrap_err();
    assert_jiminy_prog_err(&err, expected);
}
