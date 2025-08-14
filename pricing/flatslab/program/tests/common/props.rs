use inf1_pp_core::pair::Pair;
use inf1_pp_flatslab_core::{
    accounts::{Slab, SlabMut},
    pricing::FlatSlabPricing,
};
use proptest::{collection::vec, prelude::*};

const EXPECTED_ENTRY_SIZE: usize = 40;

pub fn slab_for_swap() -> impl Strategy<Value = (Vec<u8>, Pair<[u8; 32]>, FlatSlabPricing)> {
    (2usize..=69) // need at least 2 elems for swap
        .prop_flat_map(|n| vec(any::<u8>(), 32 + n * EXPECTED_ENTRY_SIZE))
        .prop_flat_map(|b| {
            let len = (b.len() - 32) / EXPECTED_ENTRY_SIZE;
            (Just(b), 0..len, 0..len)
        })
        .prop_map(|(mut b, i, o)| {
            let mut slab_mut = SlabMut::of_acc_data(&mut b).unwrap();
            let (_, entries) = slab_mut.as_mut();
            entries.0.sort_unstable_by_key(|e| *e.mint());

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
