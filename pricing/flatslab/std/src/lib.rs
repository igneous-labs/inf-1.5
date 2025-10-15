use inf1_pp_flatslab_core::{accounts::Slab, typedefs::SlabEntryPackedList};

pub mod traits;
pub mod update;

// Re-exports
pub use inf1_pp_flatslab_core::*;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct FlatSlabPricing {
    slab_acc_data: Box<[u8]>,
}

impl FlatSlabPricing {
    #[inline]
    pub const fn entries(&self) -> SlabEntryPackedList<'_> {
        match Slab::of_acc_data(&self.slab_acc_data) {
            Some(s) => s.entries(),
            None => SlabEntryPackedList::new(&[]),
        }
    }
}
