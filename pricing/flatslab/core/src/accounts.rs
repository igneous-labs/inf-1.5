use core::slice;

use crate::internal_utils::{impl_cast_from_acc_data, impl_cast_to_acc_data};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SlabEntryPacked {
    mint: [u8; 32],
    inp_fee_nanos: [u8; 4],
    out_fee_nanos: [u8; 4],
}

/// Constructors
impl SlabEntryPacked {
    pub const DEFAULT: Self = Self {
        mint: [0; 32],
        inp_fee_nanos: [0; 4],
        out_fee_nanos: [0; 4],
    };
}

impl Default for SlabEntryPacked {
    #[inline]
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// Accessors
impl SlabEntryPacked {
    #[inline]
    pub const fn mint(&self) -> &[u8; 32] {
        &self.mint
    }

    #[inline]
    pub const fn inp_fee_nanos(&self) -> i32 {
        i32::from_le_bytes(self.inp_fee_nanos)
    }

    #[inline]
    pub const fn out_fee_nanos(&self) -> i32 {
        i32::from_le_bytes(self.out_fee_nanos)
    }
}

/// Mutators
impl SlabEntryPacked {
    #[inline]
    pub const fn mint_mut(&mut self) -> &mut [u8; 32] {
        &mut self.mint
    }

    #[inline]
    pub const fn set_inp_fee_nanos(&mut self, inp_fee_nanos: i32) {
        self.inp_fee_nanos = inp_fee_nanos.to_le_bytes();
    }

    #[inline]
    pub const fn set_out_fee_nanos(&mut self, out_fee_nanos: i32) {
        self.out_fee_nanos = out_fee_nanos.to_le_bytes();
    }
}

impl_cast_from_acc_data!(SlabEntryPacked, packed);
impl_cast_to_acc_data!(SlabEntryPacked, packed);

const _ASSERT_SLAB_ENTRY_PACKED_ALIGN: () = assert!(align_of::<SlabEntryPacked>() == 1);

/// Returns element length of [`PackedList`] if acc_data is a valid one
const fn packed_list_len<T>(acc_data: &[u8]) -> Option<usize> {
    const {
        assert!(align_of::<T>() == 1);
    }

    let tlen: usize = size_of::<T>();
    if acc_data.len() % tlen != 0 {
        return None;
    }
    Some(acc_data.len() / tlen)
}

pub type SlabEntryPackedList<'a> = PackedList<'a, SlabEntryPacked>;
pub type SlabEntryPackedListMut<'a> = PackedListMut<'a, SlabEntryPacked>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PackedList<'a, T>(pub &'a [T]);

/// pointer casting "serde"
impl<'a, T> PackedList<'a, T> {
    #[inline]
    pub const fn of_acc_data(acc_data: &'a [u8]) -> Option<Self> {
        match packed_list_len::<T>(acc_data) {
            None => None,
            Some(len) => Some(Self(unsafe {
                slice::from_raw_parts(acc_data.as_ptr().cast(), len)
            })),
        }
    }

    #[inline]
    pub const fn as_acc_data(&self) -> &[u8] {
        let bytes = self.0.len() * size_of::<T>();
        unsafe { slice::from_raw_parts(self.0.as_ptr().cast(), bytes) }
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct PackedListMut<'a, T>(pub &'a mut [T]);

/// pointer casting "deserialization"
impl<'a, T> PackedListMut<'a, T> {
    #[inline]
    pub const fn of_acc_data(acc_data: &'a mut [u8]) -> Option<Self> {
        match packed_list_len::<T>(acc_data) {
            None => None,
            Some(len) => Some(Self(unsafe {
                slice::from_raw_parts_mut(acc_data.as_mut_ptr().cast(), len)
            })),
        }
    }
}

/// to immut
impl<T> PackedListMut<'_, T> {
    #[inline]
    pub const fn as_packed_list(&self) -> PackedList<'_, T> {
        PackedList(self.0)
    }
}

/// Accssors
impl SlabEntryPackedList<'_> {
    /// Returns `Err(index to insert to maintain sorted order)` if entry of mint not in list
    #[inline]
    pub fn find_by_mint(&self, mint: &[u8; 32]) -> Result<&SlabEntryPacked, usize> {
        self.0
            .binary_search_by_key(mint, |entry| *entry.mint())
            .map(|i| &self.0[i])
    }
}

/// Accssors
impl SlabEntryPackedListMut<'_> {
    /// Returns `Err(index to insert to maintain sorted order)` if entry of mint not in list
    #[inline]
    pub fn find_by_mint_mut(&mut self, mint: &[u8; 32]) -> Result<&mut SlabEntryPacked, usize> {
        self.0
            .binary_search_by_key(mint, |entry| *entry.mint())
            .map(|i| &mut self.0[i])
    }
}

// `.0` - full account data
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Slab<'a>(&'a [u8]);

/// pointer casting "serde"
impl<'a> Slab<'a> {
    #[inline]
    pub const fn of_acc_data(acc_data: &'a [u8]) -> Option<Self> {
        let (_manager, entries) = match acc_data.split_first_chunk::<32>() {
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
    pub const fn manager(&self) -> &[u8; 32] {
        match self.0.split_first_chunk::<32>() {
            // unreachable!(): inner data guaranteed to be valid at construction
            None => unreachable!(),
            Some((manager, _entries)) => manager,
        }
    }

    #[inline]
    pub const fn entries(&self) -> SlabEntryPackedList<'a> {
        // unreachable!()s: inner data guaranteed to be valid at construction
        let entries = match self.0.split_first_chunk::<32>() {
            None => unreachable!(),
            Some((_manager, entries)) => entries,
        };
        match SlabEntryPackedList::of_acc_data(entries) {
            None => unreachable!(),
            Some(list) => list,
        }
    }
}

// `.0` - full account data
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct SlabMut<'a>(&'a mut [u8]);

/// pointer casting "deser"
impl<'a> SlabMut<'a> {
    #[inline]
    pub const fn of_acc_data(acc_data: &'a mut [u8]) -> Option<Self> {
        let (_manager, entries) = match acc_data.split_first_chunk::<32>() {
            None => return None,
            Some(a) => a,
        };
        match SlabEntryPackedList::of_acc_data(entries) {
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
    /// Returns `(manager, entries)`
    #[inline]
    pub const fn as_mut(&mut self) -> (&mut [u8; 32], SlabEntryPackedListMut<'_>) {
        // unreachable!()s: inner data guaranteed to be valid at construction
        match self.0.split_first_chunk_mut::<32>() {
            None => unreachable!(),
            Some((manager, entries)) => (
                manager,
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

    const EXPECTED_ENTRY_SIZE: usize = 40;

    prop_compose! {
        fn rand_slab_params()
        (
            data in vec(any::<u8>(), 0..=8192),
        )
        (
            edit_idx in if data.len() < 32 + EXPECTED_ENTRY_SIZE {
                Just(None).boxed()
            } else {
                (0..=(data.len() - 32) / EXPECTED_ENTRY_SIZE).prop_map(Some).boxed()
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
            let should_be_valid = data.len() > 32 && (data.len() - 32) % EXPECTED_ENTRY_SIZE == 0;
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
            prop_assert_eq!(*s.manager(), SET_MAN_TO);
            let e = s.entries().0[edit_idx];
            prop_assert_eq!(e.inp_fee_nanos(), SET_INP_FEE_NANOS_TO);
            prop_assert_eq!(e.out_fee_nanos(), SET_OUT_FEE_NANOS_TO);
        }
    }
}
