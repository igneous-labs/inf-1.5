use inf1_pp_reserve_v2_core::typedefs::{FeeEntry, FeeEntryGen, FeeEntryNanos};

use crate::{assert_list_changes, gas_diff_zip_assert, Diff, ListChange};

pub type DiffsFeeEntry = FeeEntryGen<Diff<[u8; 32]>, FeeEntryNanos<Diff<u32>>>;

/// `bef` and `aft` are the raw account data bytes before and after execution.
///
/// Returns [pricing_state_bef, pricing_state_aft]
pub fn assert_diffs_pricing_state(
    (admin, entries): (Diff<[u8; 32]>, Vec<ListChange<DiffsFeeEntry, FeeEntry>>),
    (bef_admin, bef_entries): (&[u8; 32], &[FeeEntry]),
    (aft_admin, aft_entries): (&[u8; 32], &[FeeEntry]),
) {
    admin.assert(bef_admin, aft_admin);
    assert_list_changes(&entries, bef_entries, aft_entries, |diff, bef, aft| {
        diff.mint.assert(&bef.mint, &aft.mint);
        gas_diff_zip_assert!(diff.nanos, bef.nanos, aft.nanos);
    });
}
