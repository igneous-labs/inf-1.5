use core::mem::size_of;

use crate::typedefs::{SlabEntryPacked, SlabEntryPackedList, SlabEntryPackedListMut};

// `.0` - full account data
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Slab<'a>(&'a [u8]);

/// pointer casting "serde"
impl<'a> Slab<'a> {
    #[inline]
    pub const fn of_acc_data(acc_data: &'a [u8]) -> Option<Self> {
        let (_admin, entries) = match acc_data.split_first_chunk::<32>() {
            None => return None,
            Some(a) => a,
        };
        match SlabEntryPackedList::of_acc_data(entries) {
            None => None,
            Some(_) => Some(Self(acc_data)),
        }
    }

    #[inline]
    pub const fn as_acc_data(&self) -> &[u8] {
        self.0
    }
}

/// Accessors
impl<'a> Slab<'a> {
    #[inline]
    pub const fn admin(&self) -> &[u8; 32] {
        match self.0.split_first_chunk::<32>() {
            // unreachable!(): inner data guaranteed to be valid at construction
            None => unreachable!(),
            Some((admin, _entries)) => admin,
        }
    }

    #[inline]
    pub const fn entries(&self) -> SlabEntryPackedList<'a> {
        // unreachable!()s: inner data guaranteed to be valid at construction
        let entries = match self.0.split_first_chunk::<32>() {
            None => unreachable!(),
            Some((_admin, entries)) => entries,
        };
        match SlabEntryPackedList::of_acc_data(entries) {
            None => unreachable!(),
            Some(list) => list,
        }
    }
}

/// Account len utils
impl Slab<'_> {
    #[inline]
    pub const fn account_size(n_entries: usize) -> usize {
        32 + n_entries * size_of::<SlabEntryPacked>()
    }

    // exact same fn as `Self::account_size`
    #[inline]
    pub const fn entry_byte_offset(idx: usize) -> usize {
        32 + idx * size_of::<SlabEntryPacked>()
    }
}

// `.0` - full account data
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct SlabMut<'a>(&'a mut [u8]);

/// pointer casting "deser"
impl<'a> SlabMut<'a> {
    #[inline]
    pub const fn of_acc_data(acc_data: &'a mut [u8]) -> Option<Self> {
        match Slab::of_acc_data(acc_data) {
            None => None,
            Some(_) => Some(Self(acc_data)),
        }
    }
}

/// to immut
impl SlabMut<'_> {
    #[inline]
    pub const fn as_slab(&self) -> Slab<'_> {
        Slab(self.0)
    }
}

/// Mutators
impl SlabMut<'_> {
    /// Returns `(admin, entries)`
    #[inline]
    pub const fn as_mut(&mut self) -> (&mut [u8; 32], SlabEntryPackedListMut<'_>) {
        // unreachable!()s: inner data guaranteed to be valid at construction
        match self.0.split_first_chunk_mut::<32>() {
            None => unreachable!(),
            Some((admin, entries)) => (
                admin,
                match SlabEntryPackedListMut::of_acc_data(entries) {
                    None => unreachable!(),
                    Some(list) => list,
                },
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use proptest::{collection::vec, prelude::*};

    use super::*;

    prop_compose! {
        fn rand_slab_params()
        (
            data in vec(any::<u8>(), 0..=8192),
        )
        (
            edit_idx in if data.len() < Slab::account_size(1) {
                Just(None).boxed()
            } else {
                // entry array length is at least 1 from check above,
                // so no 0..0 empty range possible
                (0..(data.len() - 32) / size_of::<SlabEntryPacked>()).prop_map(Some).boxed()
            },
            data in Just(data),
        ) -> (Vec<u8>, Option<usize>) {
            (data, edit_idx)
        }
    }

    proptest! {
        #[test]
        fn slab_general_mutate_then_check((mut data, edit_idx) in rand_slab_params()) {
            const SET_MAN_TO: [u8; 32] = [1u8; 32];
            const SET_MINT_TO: [u8; 32] = [69u8; 32];
            const SET_INP_FEE_NANOS_TO: i32 = i32::MIN;
            const SET_OUT_FEE_NANOS_TO: i32 = i32::MAX;

            let deser = Slab::of_acc_data(&data);
            let should_be_valid = data.len() >= 32 && (data.len() - 32) % size_of::<SlabEntryPacked>() == 0;
            if !should_be_valid {
                prop_assert!(deser.is_none());
                return Ok(());
            }

            // valid slab

            let edit_idx = match edit_idx {
                Some(i) => i,
                None => return Ok(()),
            };
            let mut sm = SlabMut::of_acc_data(data.as_mut_slice()).unwrap();

            let (man, entries) = sm.as_mut();

            *man = SET_MAN_TO;
            let e = &mut entries.0[edit_idx];
            *e.mint_mut() = SET_MINT_TO;
            e.set_inp_fee_nanos(SET_INP_FEE_NANOS_TO);
            e.set_out_fee_nanos(SET_OUT_FEE_NANOS_TO);

            let s = Slab::of_acc_data(&data).unwrap();
            prop_assert_eq!(*s.admin(), SET_MAN_TO);
            let e = s.entries().0[edit_idx];
            prop_assert_eq!(e.inp_fee_nanos(), SET_INP_FEE_NANOS_TO);
            prop_assert_eq!(e.out_fee_nanos(), SET_OUT_FEE_NANOS_TO);
        }
    }
}
