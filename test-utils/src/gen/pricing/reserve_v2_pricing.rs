use std::ops::Range;

use inf1_pp_reserve_v2_core::{
    accounts::pricing_state_account_size,
    keys::CONST_KEYS_OWNED,
    typedefs::{
        FeeEntry, FeeEntryNanos, FeeEntryNanosDestr, FeeEntryPacked, FeeNanos, ThresholdNanos,
    },
};
use jiminy_sysvar_rent::Rent;
use proptest::collection::vec as prop_vec;
use proptest::prelude::*;
use solana_account::Account;
use solana_pubkey::Pubkey;

/// `entries` must be sorted by mint
pub fn mock_reserve_v2_pricing_state_account(admin: [u8; 32], entries: &[FeeEntry]) -> Account {
    let size = pricing_state_account_size(entries.len());
    let mut data = vec![0u8; size];
    data[..32].copy_from_slice(&admin);
    let entry_data = &mut data[32..];
    for (i, entry) in entries.iter().enumerate() {
        let packed = FeeEntryPacked::from(*entry);
        let entry_bytes = packed.as_acc_data_arr();
        let start = i * entry_bytes.len();
        entry_data[start..start + entry_bytes.len()].copy_from_slice(entry_bytes);
    }
    Account {
        lamports: Rent::default().min_balance(data.len()),
        data,
        owner: Pubkey::new_from_array(*CONST_KEYS_OWNED.program()),
        executable: false,
        rent_epoch: u64::MAX,
    }
}

/// Generates a valid [`FeeEntry`] for the given mint with
/// random fees (sorted: base <= threshold <= max).
pub fn any_fee_entry(mint: [u8; 32]) -> impl Strategy<Value = FeeEntry> {
    (
        0..=FeeNanos::MAX.get(),
        0..=FeeNanos::MAX.get(),
        0..=FeeNanos::MAX.get(),
        0..=FeeNanos::MAX.get(),
        ThresholdNanos::MIN.get()..=ThresholdNanos::MAX.get(),
    )
        .prop_map(move |(a, b, c, of, t)| {
            let mut fee = [a, b, c];
            fee.sort_unstable();
            FeeEntry {
                mint,
                threshold_nanos: t,
                fee_nanos: FeeEntryNanos::from_destr(FeeEntryNanosDestr {
                    base_fee: fee[0],
                    threshold_fee: fee[1],
                    max_fee: fee[2],
                    output_fee: of,
                }),
            }
        })
}

/// Generates a valid [`ThresholdNanos`] (1..=999_999_999).
pub fn any_threshold_nanos() -> impl Strategy<Value = ThresholdNanos> {
    (ThresholdNanos::MIN.get()..=ThresholdNanos::MAX.get())
        .prop_map(|t| ThresholdNanos::new(t).unwrap())
}

pub fn any_invalid_threshold_nanos() -> impl Strategy<Value = u32> {
    prop_oneof![Just(0), ThresholdNanos::MAX.get() + 1..=u32::MAX]
}

/// Generates a valid [`FeeNanos`] (0..=1_000_000_000).
pub fn any_fee_nanos() -> impl Strategy<Value = FeeNanos> {
    (0..=FeeNanos::MAX.get()).prop_map(|n| FeeNanos::new(n).unwrap())
}

pub fn any_invalid_fee_nanos() -> impl Strategy<Value = u32> {
    FeeNanos::MAX.get() + 1..=u32::MAX
}

/// Generates a valid [`FeeEntryNanos<FeeNanos>`] with `base <= threshold <= max`.
pub fn any_fee_entry_nanos() -> impl Strategy<Value = FeeEntryNanos<FeeNanos>> {
    [(); 4].map(|_| any_fee_nanos()).prop_map(|[a, b, c, of]| {
        let mut if_knots = [a, b, c];
        if_knots.sort_unstable();
        FeeEntryNanos::from_destr(FeeEntryNanosDestr {
            base_fee: if_knots[0],
            threshold_fee: if_knots[1],
            max_fee: if_knots[2],
            output_fee: of,
        })
    })
}

/// A valid pricing state account with admin and 2 entries (LP_MINT, WSOL_MINT).
fn any_extra_entry() -> impl Strategy<Value = FeeEntry> {
    any::<[u8; 32]>().prop_flat_map(any_fee_entry)
}

/// Generates a valid (admin, entries) pair for a pricing state.
/// `extra_count` controls how many random extra entries are added
/// beyond the required `LP_MINT` and `WSOL_MINT`. Entries are sorted.
pub fn any_reserve_v2_pricing_state(
    extra_count: Range<usize>,
) -> impl Strategy<Value = ([u8; 32], Vec<FeeEntry>)> {
    (
        any::<[u8; 32]>(),
        any_fee_entry(*CONST_KEYS_OWNED.lp_mint()),
        any_fee_entry(*CONST_KEYS_OWNED.wsol_mint()),
        prop_vec(any_extra_entry(), extra_count),
    )
        .prop_map(|(admin, lp_entry, wsol_entry, extra)| {
            let mut entries: Vec<_> = [vec![lp_entry, wsol_entry], extra].concat();
            entries.sort_unstable_by_key(|e| e.mint);
            (admin, entries)
        })
}
