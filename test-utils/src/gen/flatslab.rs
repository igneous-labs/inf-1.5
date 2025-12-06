use std::{array, collections::HashSet, iter::once};

use inf1_pp_flatslab_core::{
    instructions::pricing::FlatSlabPpAccs,
    typedefs::{FeeNanos, SlabEntryPacked},
};
use proptest::{collection::vec, prelude::*};

use crate::{flatslab_acc_data, mock_flatslab_slab, AccountMap};

/// See [`reasonable_flatslab_data_strat`]
pub fn reasonable_flatslab_strat_for_mints(
    mints: HashSet<[u8; 32]>,
) -> impl Strategy<Value = (FlatSlabPpAccs, AccountMap)> {
    reasonable_flatslab_data_strat(mints)
        .prop_map(mock_flatslab_slab)
        .prop_map(|slab| {
            (
                FlatSlabPpAccs::MAINNET,
                once(((*FlatSlabPpAccs::MAINNET.0.slab()).into(), slab)).collect(),
            )
        })
}

/// Generates slabs that have individual fees in [0, 45%],
/// so fee ranges from 0-90%.
///
/// Sets admin to all zeros
pub fn reasonable_flatslab_data_strat(mints: HashSet<[u8; 32]>) -> impl Strategy<Value = Vec<u8>> {
    vec(
        array::from_fn(|_| reasonable_flatslab_fee_nanos_strat()),
        mints.len(),
    )
    .prop_map(move |fees| {
        let entries = fees
            .into_iter()
            .zip(mints.clone())
            .map(|([inp, out], mint)| {
                let mut raw = SlabEntryPacked::DEFAULT;
                *raw.mint_mut() = mint;
                raw.set_inp_fee_nanos(inp);
                raw.set_out_fee_nanos(out);
                raw
            });
        flatslab_acc_data([0u8; 32], entries)
    })
}

fn reasonable_flatslab_fee_nanos_strat() -> impl Strategy<Value = FeeNanos> {
    (0..=450_000_000).prop_map(|n| FeeNanos::new(n).unwrap())
}
