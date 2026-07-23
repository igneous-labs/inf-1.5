use crate::typedefs::{FeeEntry, FeeEntryList, FeeEntryListMut, FeeEntryPackedList};

/// Returns `(admin, entries)`
#[inline]
pub fn pricing_state_of_acc_data(data: &[u8]) -> Option<(&[u8; 32], FeeEntryList<'_>)> {
    let (admin, entries) = data.split_first_chunk::<32>()?;
    FeeEntryList::of_acc_data(entries).map(|entries| (admin, entries))
}

/// Returns `(admin, entries)`
#[inline]
pub fn pricing_state_of_acc_data_mut(
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

#[cfg(test)]
mod tests {
    use proptest::{collection::vec, prelude::*};

    use crate::typedefs::{FeeEntryNanos, FeeEntryNanosDestr, FeeNanos, ThresholdNanos};

    use super::*;

    prop_compose! {
        fn rand_pricing_state_params()
        (
            data in vec(any::<u8>(), 0..=8192),
        )
        (
            edit_idx in if data.len() < pricing_state_account_size(1) {
                Just(None).boxed()
            } else {
                // entry array length is at least 1 from check above,
                // so no 0..0 empty range possible
                (0..(data.len() - 32) / size_of::<FeeEntry>()).prop_map(Some).boxed()
            },
            data in Just(data),
        ) -> (Vec<u8>, Option<usize>) {
            (data, edit_idx)
        }
    }

    proptest! {
        #[test]
        fn pricing_state_general_mutate_then_check((mut data, edit_idx) in rand_pricing_state_params()) {
            const SET_ADMIN_TO: [u8; 32] = [1u8; 32];
            const SET_MINT_TO: [u8; 32] = [69u8; 32];
            const SET_BASE_FEE_NANOS_TO: FeeNanos = FeeNanos::ZERO;
            const SET_THRESHOLD_NANOS_TO: ThresholdNanos = ThresholdNanos::MIN;
            const SET_THRESHOLD_FEE_NANOS_TO: FeeNanos = FeeNanos::ZERO;
            const SET_MAX_FEE_NANOS_TO: FeeNanos = FeeNanos::MAX;
            const SET_OUTPUT_FEE_NANOS_TO: FeeNanos = FeeNanos::ZERO;

            let deser = pricing_state_of_acc_data(&data);
            #[allow(clippy::manual_is_multiple_of)]
            let should_be_valid = data.len() >= 32
                && (data.len() - 32) % size_of::<FeeEntry>() == 0
                && (data[32..].as_ptr() as usize) % align_of::<FeeEntry>() == 0;
            if !should_be_valid {
                prop_assert!(deser.is_none());
                return Ok(());
            }

            let edit_idx = match edit_idx {
                Some(i) => i,
                None => return Ok(()),
            };
            let (admin, entries) = pricing_state_of_acc_data_mut(data.as_mut_slice()).unwrap();

            *admin = SET_ADMIN_TO;
            let e = &mut entries.0[edit_idx];
            e.mint = SET_MINT_TO;
            e.nanos = FeeEntryNanos::const_from_destr(FeeEntryNanosDestr {
                base_fee: SET_BASE_FEE_NANOS_TO.get(),
                threshold: SET_THRESHOLD_NANOS_TO.get(),
                threshold_fee: SET_THRESHOLD_FEE_NANOS_TO.get(),
                max_fee: SET_MAX_FEE_NANOS_TO.get(),
                output_fee: SET_OUTPUT_FEE_NANOS_TO.get(),
            });

            let (admin, entries) = pricing_state_of_acc_data(&data).unwrap();
            prop_assert_eq!(*admin, SET_ADMIN_TO);
            let e = entries.0[edit_idx];
            prop_assert_eq!(e.mint, SET_MINT_TO);
            prop_assert_eq!(e.nanos.base_fee_nanos(), SET_BASE_FEE_NANOS_TO);
            prop_assert_eq!(e.nanos.threshold_nanos(), SET_THRESHOLD_NANOS_TO);
            prop_assert_eq!(e.nanos.threshold_fee_nanos(), SET_THRESHOLD_FEE_NANOS_TO);
            prop_assert_eq!(e.nanos.max_fee_nanos(), SET_MAX_FEE_NANOS_TO);
            prop_assert_eq!(e.nanos.output_fee_nanos(), SET_OUTPUT_FEE_NANOS_TO);
        }
    }
}
