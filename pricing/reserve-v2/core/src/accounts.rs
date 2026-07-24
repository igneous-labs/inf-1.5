use crate::typedefs::{FeeEntry, FeeEntryList, FeeEntryListMut, FeeEntryPackedList};

/// Returns `(admin, entries)`
///
/// # Safety
/// Rules of [`FeeEntryList::of_acc_data`] apply
#[inline]
pub unsafe fn pricing_state_of_acc_data(data: &[u8]) -> Option<(&[u8; 32], FeeEntryList<'_>)> {
    let (admin, entries) = data.split_first_chunk::<32>()?;
    FeeEntryList::of_acc_data(entries).map(|entries| (admin, entries))
}

/// Returns `(admin, entries)`
///
/// # Safety
/// Rules of [`FeeEntryListMut::of_acc_data`] apply
#[inline]
pub unsafe fn pricing_state_of_acc_data_mut(
    data: &mut [u8],
) -> Option<(&mut [u8; 32], FeeEntryListMut<'_>)> {
    let (admin, entries) = data.split_first_chunk_mut::<32>()?;
    FeeEntryListMut::of_acc_data(entries).map(|entries| (admin, entries))
}

/// Returns `(admin, entries)`
#[inline]
pub fn pricing_state_of_acc_data_packed(
    data: &[u8],
) -> Option<(&[u8; 32], FeeEntryPackedList<'_>)> {
    let (admin, entries) = data.split_first_chunk::<32>()?;
    FeeEntryPackedList::of_acc_data(entries).map(|entries| (admin, entries))
}

#[inline]
pub const fn pricing_state_account_size(n_entries: usize) -> usize {
    32 + n_entries * size_of::<FeeEntry>()
}
