use inf1_pp_core::pair::Pair;
use inf1_pp_flatslab_core::{
    accounts::Slab,
    keys::{LP_MINT_ID, SLAB_ID},
    pricing::FlatSlabPricing,
    typedefs::{SlabEntryPacked, SlabEntryPackedList},
};
use proptest::{collection::vec, prelude::*};

/// Balance between large size to cover cases and small size for proptest exec speed
pub const MAX_MINTS: usize = 10;

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
        .prop_flat_map(|n| vec(any::<u8>(), Slab::account_size(n)))
        .prop_flat_map(|b| {
            let len = Slab::of_acc_data(&b).unwrap().entries().0.len();
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

/// Returns `(slab_account_data, non_lp_mint, lp_entry, non_lp_mint_entry)`
pub fn slab_for_liq(
    max_mints: usize,
) -> impl Strategy<Value = (Vec<u8>, [u8; 32], SlabEntryPacked, SlabEntryPacked)> {
    slab_for_swap(max_mints)
        .prop_flat_map(|tup| (Just(tup), any::<[u8; EXPECTED_ENTRY_SIZE]>()))
        .prop_map(|((mut slab_data, Pair { inp, .. }, _), mut lp_entry)| {
            lp_entry[..32].copy_from_slice(&LP_MINT_ID);
            slab_data.extend(lp_entry);

            let slab_data = to_rand_slab_data(slab_data);
            let entries = Slab::of_acc_data(&slab_data).unwrap().entries();
            // just use randomly generated `inp` mint as the non LP mint
            let [lp_entry, other_entry] =
                [LP_MINT_ID, inp].map(|mint| *entries.find_by_mint(&mint).unwrap());

            (slab_data, inp, lp_entry, other_entry)
        })
}

pub fn non_slab_pks() -> impl Strategy<Value = [u8; 32]> {
    any::<[u8; 32]>().prop_filter("Must not be slab ID", |v| *v != SLAB_ID)
}
