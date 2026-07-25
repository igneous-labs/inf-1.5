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
    mock_reserve_v2_pricing_state_account, mock_token_acc, pool_state_v2_account, raw_token_acc,
    AccountMap,
};
use solana_account::Account;
use solana_pubkey::Pubkey;

mod exact_out;

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
