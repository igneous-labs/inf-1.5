use core::mem::size_of;

use crate::typedefs::{FeeEntry, FeeEntryList, FeeEntryListMut};

// `.0` - admin
// `.1` - fee entries
/// # Invariants
/// - [`crate::keys::LP_MINT`] is always an entry in `PricingState`
/// - [`crate::keys::WSOL_MINT`] is always an entry in `PricingState`
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PricingState<'a>(&'a [u8; 32], FeeEntryList<'a>);

/// pointer casting "serde"
impl<'a> PricingState<'a> {
    #[inline]
    pub fn of_acc_data(acc_data: &'a [u8]) -> Option<Self> {
        let (admin, entries) = acc_data.split_first_chunk::<32>()?;
        FeeEntryList::of_acc_data(entries).map(|entries| Self(admin, entries))
    }
}

/// Accessors
impl<'a> PricingState<'a> {
    #[inline]
    pub const fn admin(&self) -> &[u8; 32] {
        self.0
    }

    #[inline]
    pub const fn entries(&self) -> FeeEntryList<'a> {
        self.1
    }
}

/// Account len utils
impl PricingState<'_> {
    #[inline]
    pub const fn account_size(n_entries: usize) -> usize {
        32 + n_entries * size_of::<FeeEntry>()
    }

    #[inline]
    pub const fn entry_byte_offset(idx: usize) -> usize {
        32 + idx * size_of::<FeeEntry>()
    }
}

// `.0` - admin
// `.1` - fee entries
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct PricingStateMut<'a>(&'a mut [u8; 32], FeeEntryListMut<'a>);

/// pointer casting "deser"
impl<'a> PricingStateMut<'a> {
    #[inline]
    pub fn of_acc_data(acc_data: &'a mut [u8]) -> Option<Self> {
        let (admin, entries) = acc_data.split_first_chunk_mut::<32>()?;
        FeeEntryListMut::of_acc_data(entries).map(|entries| Self(admin, entries))
    }
}

/// to immut
impl<'a> PricingStateMut<'a> {
    #[inline]
    pub const fn as_pricing_state(&self) -> PricingState<'_> {
        PricingState(self.0, self.1.as_list())
    }
}

/// Mutators
impl PricingStateMut<'_> {
    /// Returns `(admin, entries)`
    #[inline]
    pub fn as_mut(&mut self) -> (&mut [u8; 32], FeeEntryListMut<'_>) {
        (&mut *self.0, FeeEntryListMut(&mut *self.1 .0))
    }
}

#[cfg(test)]
mod tests {
    use proptest::{collection::vec, prelude::*};

    use crate::typedefs::{FeeNanos, ThresholdNanos};

    use super::*;

    prop_compose! {
        fn rand_pricing_state_params()
        (
            data in vec(any::<u8>(), 0..=8192),
        )
        (
            edit_idx in if data.len() < PricingState::account_size(1) {
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

            let deser = PricingState::of_acc_data(&data);
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
            let mut state_mut = PricingStateMut::of_acc_data(data.as_mut_slice()).unwrap();

            let (admin, entries) = state_mut.as_mut();

            *admin = SET_ADMIN_TO;
            let e = &mut entries.0[edit_idx];
            *e.mint_mut() = SET_MINT_TO;
            e.set_base_fee_nanos(SET_BASE_FEE_NANOS_TO);
            e.set_threshold_nanos(SET_THRESHOLD_NANOS_TO);
            e.set_threshold_fee_nanos(SET_THRESHOLD_FEE_NANOS_TO);
            e.set_max_fee_nanos(SET_MAX_FEE_NANOS_TO);
            e.set_output_fee_nanos(SET_OUTPUT_FEE_NANOS_TO);

            let state = PricingState::of_acc_data(&data).unwrap();
            prop_assert_eq!(*state.admin(), SET_ADMIN_TO);
            let e = state.entries().0[edit_idx];
            prop_assert_eq!(*e.mint(), SET_MINT_TO);
            prop_assert_eq!(e.base_fee_nanos(), SET_BASE_FEE_NANOS_TO);
            prop_assert_eq!(e.threshold_nanos(), SET_THRESHOLD_NANOS_TO);
            prop_assert_eq!(e.threshold_fee_nanos(), SET_THRESHOLD_FEE_NANOS_TO);
            prop_assert_eq!(e.max_fee_nanos(), SET_MAX_FEE_NANOS_TO);
            prop_assert_eq!(e.output_fee_nanos(), SET_OUTPUT_FEE_NANOS_TO);
        }
    }
}
