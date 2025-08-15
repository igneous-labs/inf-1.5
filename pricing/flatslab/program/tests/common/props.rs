use inf1_pp_core::pair::Pair;
use inf1_pp_flatslab_core::{
    accounts::Slab, keys::SLAB_ID, pricing::FlatSlabPricing, typedefs::SlabEntryPackedList,
};
use proptest::{collection::vec, prelude::*};

pub const SLAB_HEADER_SIZE: usize = 32;
pub const EXPECTED_ENTRY_SIZE: usize = 40;

pub fn to_rand_slab_data(rand_data: Vec<u8>) -> Vec<u8> {
    let slab = Slab::of_acc_data(&rand_data).unwrap();
    // can probably use itertools dedup to avoid a new vec here
    let mut entries = Vec::from(slab.entries().0);
    entries.sort_unstable_by_key(|e| *e.mint());
    entries.dedup_by_key(|e| *e.mint());
    slab.as_acc_data()[..SLAB_HEADER_SIZE]
        .iter()
        .chain(SlabEntryPackedList::new(&entries).as_acc_data())
        .copied()
        .collect()
}

pub fn slab_for_swap(
    max_mints: usize,
) -> impl Strategy<Value = (Vec<u8>, Pair<[u8; 32]>, FlatSlabPricing)> {
    (2usize..=max_mints) // need at least 2 elems for swap
        .prop_flat_map(|n| vec(any::<u8>(), SLAB_HEADER_SIZE + n * EXPECTED_ENTRY_SIZE))
        .prop_flat_map(|b| {
            let len = (b.len() - SLAB_HEADER_SIZE) / EXPECTED_ENTRY_SIZE;
            (Just(b), 0..len, 0..len)
        })
        .prop_map(|(b, i, o)| {
            let b = to_rand_slab_data(b);

            let slab = Slab::of_acc_data(&b).unwrap();
            let entries = slab.entries();
            let [(inp, inp_fee_nanos, _), (out, _, out_fee_nanos)] = [i, o].map(|idx| {
                let entry = entries.0[idx];
                (*entry.mint(), entry.inp_fee_nanos(), entry.out_fee_nanos())
            });
            (
                b,
                Pair { inp, out },
                FlatSlabPricing {
                    inp_fee_nanos,
                    out_fee_nanos,
                },
            )
        })
}

pub fn non_slab_pks() -> impl Strategy<Value = [u8; 32]> {
    any::<[u8; 32]>().prop_filter("Must not be slab ID", |v| *v != SLAB_ID)
}
