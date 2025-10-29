//! FIXME: refactor tests:
//! - move generation to test-utils crate
//! - change generation from "account to args" style to
//!   "args to account" style, like everything else in test-utils gen/ folder

use std::ops::RangeInclusive;

use inf1_pp_core::pair::Pair;
use inf1_pp_flatslab_core::{
    accounts::{Slab, SlabMut},
    keys::{LP_MINT_ID, SLAB_ID},
    pricing::FlatSlabSwapPricing,
    typedefs::{FeeNanos, SlabEntryPacked, SlabEntryPackedList},
};
use inf1_pp_flatslab_program::SYS_PROG_ID;
use proptest::{collection::vec, prelude::*};

/// Balance between large size to cover cases and small size for proptest exec speed
pub const MAX_MINTS: usize = 10;

const SLAB_HEADER_SIZE: usize = 32;

pub fn clean_valid_slab(mut rand_data: Vec<u8>) -> Vec<u8> {
    let mut slab = SlabMut::of_acc_data(&mut rand_data).unwrap();

    // LP_MINT_ID always on slab invariant
    let entries = slab.as_mut().1 .0;
    if !entries.iter().any(|e| *e.mint() == LP_MINT_ID) {
        *entries[0].mint_mut() = LP_MINT_ID;
    }

    let slab = slab.as_slab();

    // can probably use itertools dedup to avoid a new vec here
    let mut entries = Vec::from(slab.entries().0);
    entries.sort_unstable_by_key(|e| *e.mint());
    entries.dedup_by_key(|e| *e.mint());

    // enforce FeeNanos range invariant by clamping values
    entries.iter_mut().for_each(|e| {
        let _ = [
            (
                e.inp_fee_nanos().get(),
                SlabEntryPacked::set_inp_fee_nanos as fn(&mut SlabEntryPacked, FeeNanos),
            ),
            (e.out_fee_nanos().get(), SlabEntryPacked::set_out_fee_nanos),
        ]
        .map(|(val, setter)| {
            if val < *FeeNanos::MIN {
                setter(e, FeeNanos::MIN);
            } else if val > *FeeNanos::MAX {
                setter(e, FeeNanos::MAX);
            }
        });
    });

    slab.as_acc_data()[..SLAB_HEADER_SIZE]
        .iter()
        .chain(SlabEntryPackedList::new(&entries).as_acc_data())
        .copied()
        .collect()
}

/// `mints_range` does NOT include LP_MINT; a LP mint entry will be automatically generated
pub fn slab_data(mints_range: RangeInclusive<usize>) -> impl Strategy<Value = Vec<u8>> {
    mints_range
        .prop_flat_map(|n| vec(any::<u8>(), Slab::account_size(n + 1))) // +1 for LP_MINT
        .prop_map(clean_valid_slab)
}

pub fn slab_for_swap(
    max_mints: usize,
) -> impl Strategy<Value = (Vec<u8>, Pair<[u8; 32]>, FlatSlabSwapPricing)> {
    slab_data(2usize..=max_mints) // need at least 2 mints for swap
        .prop_flat_map(|b| {
            let entries = Slab::of_acc_data(&b).unwrap().entries().0;
            let len = entries.len();
            let lp_idx = entries
                .iter()
                .position(|e| *e.mint() == LP_MINT_ID)
                .unwrap();
            let non_lp_idxs = (0..len).prop_filter("Must not be LP mint", move |i| *i != lp_idx);
            (Just(b), non_lp_idxs.clone(), non_lp_idxs)
        })
        .prop_map(|(b, i, o)| {
            let slab = Slab::of_acc_data(&b).unwrap();
            let entries = slab.entries();
            let [(inp, inp_fee_nanos, _), (out, _, out_fee_nanos)] = [i, o].map(|idx| {
                let entry = entries.0[idx];
                (*entry.mint(), entry.inp_fee_nanos(), entry.out_fee_nanos())
            });
            (
                b,
                Pair { inp, out },
                FlatSlabSwapPricing {
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
    slab_for_swap(max_mints).prop_map(|(slab_data, Pair { inp, .. }, _)| {
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

/// Not slab, not system prog
pub fn rand_unknown_pk() -> impl Strategy<Value = [u8; 32]> {
    non_slab_pks().prop_filter("Must not be sys prog", |v| *v != SYS_PROG_ID)
}
