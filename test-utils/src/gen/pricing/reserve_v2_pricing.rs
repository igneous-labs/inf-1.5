use inf1_pp_reserve_v2_core::{
    accounts::pricing_state_account_size,
    keys::CONST_KEYS_OWNED,
    typedefs::{
        FeeEntry, FeeEntryNanos, FeeEntryNanosDestr, FeeEntryPacked, FeeNanos, ThresholdNanos,
    },
};
use proptest::prelude::*;
use solana_account::Account;
use solana_pubkey::Pubkey;

/// `entries` must be sorted by mint
pub fn mock_reserve_v2_pricing_state_account(
    admin: [u8; 32],
    entries: &[FeeEntry],
    owner: Pubkey,
) -> Account {
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
        lamports: 1_000_000_000,
        data,
        owner,
        executable: false,
        rent_epoch: u64::MAX,
    }
}

/// Generates a valid [`FeeEntry`] for the given mint with
/// random fees (sorted: base <= threshold <= max).
fn any_fee_entry(mint: [u8; 32]) -> impl Strategy<Value = FeeEntry> {
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
                nanos: FeeEntryNanos::from_destr(FeeEntryNanosDestr {
                    base_fee: fee[0],
                    threshold: t,
                    threshold_fee: fee[1],
                    max_fee: fee[2],
                    output_fee: of,
                }),
            }
        })
}

/// A valid pricing state account with admin and 2 entries (LP_MINT, WSOL_MINT).
pub fn any_reserve_v2_pricing_state() -> impl Strategy<Value = (Account, [u8; 32])> {
    (
        any::<[u8; 32]>(),
        any_fee_entry(*CONST_KEYS_OWNED.lp_mint()),
        any_fee_entry(*CONST_KEYS_OWNED.wsol_mint()),
    )
        .prop_map(|(admin, lp_entry, wsol_entry)| {
            let mut entries = vec![lp_entry, wsol_entry];
            entries.sort_unstable_by_key(|e| e.mint);
            let owner = Pubkey::new_from_array(*CONST_KEYS_OWNED.program());
            let acc = mock_reserve_v2_pricing_state_account(admin, &entries, owner);
            (acc, admin)
        })
}
