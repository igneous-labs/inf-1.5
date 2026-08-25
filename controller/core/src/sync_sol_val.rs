use crate::{
    accounts::pool_state::PoolStateV2,
    typedefs::{
        pool_sv::{PoolSvLamports, PoolSvMutRefs},
        snap::SnapU64,
    },
    yields::update::UpdateYield,
};

/// Sync SOL value of a single LST
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SyncSolVal {
    /// Snapshot of lst_state sol value across time to determine change
    pub lst_sol_val: SnapU64,

    pub curr_slot: u64,
}

impl SyncSolVal {
    /// # Returns
    /// New pool total SOL value.
    /// `None` on overflow
    ///
    /// This is rly just a wrapper for return
    /// `old_pool_total_sol_value - self.lst_sol_val.old() + self.lst_sol_val.new()`
    #[inline]
    pub const fn exec(self, old_pool_total_sol_value: u64) -> Option<u64> {
        let Self { lst_sol_val, .. } = self;
        let sub_old = match old_pool_total_sol_value.checked_sub(*lst_sol_val.old()) {
            None => return None,
            Some(x) => x,
        };
        sub_old.checked_add(*lst_sol_val.new())
    }
}

impl PoolSvLamports {
    /// Applies a [`SyncSolVal`] followed by an [`UpdateYield`] based on the changes
    /// the sync made.
    #[inline]
    pub const fn aft_ssv_uy(self, sync: &SyncSolVal) -> Option<Self> {
        let new_total_sol_value = match sync.exec(*self.total()) {
            None => return None,
            Some(x) => x,
        };
        UpdateYield {
            new_total_sol_value,
            old: self,
        }
        .exec()
    }
}

impl PoolStateV2 {
    /// Applies a [`SyncSolVal`] followed by an [`UpdateYield`] based on the changes
    /// the sync made.
    ///
    /// Assumes INF mint supply did not change
    #[inline]
    pub fn apply_ssv_uy(&mut self, sync: &SyncSolVal) -> Option<&mut Self> {
        let new = PoolSvLamports::from_pool_state_v2(self).aft_ssv_uy(sync)?;
        PoolSvMutRefs::from_pool_state_v2(self).update(new);
        if sync.lst_sol_val.old() != sync.lst_sol_val.new() {
            self.last_release_slot = sync.curr_slot;
        }
        Some(self)
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use crate::typedefs::snap::SnapDestr;

    use super::*;

    fn any_curr_slot_ssv_strat(lst_sol_val: SnapU64) -> impl Strategy<Value = SyncSolVal> {
        any::<u64>().prop_map(move |curr_slot| SyncSolVal {
            lst_sol_val,
            curr_slot,
        })
    }

    /// any PoolStateV2 compatible with given SyncSolVal:
    /// - total lamports >= lst_sol_val.old
    /// - total_lamports <= u64::MAX - lst_sol_val.new + lst_sol_val.old (so that sync exec does not overflow)
    /// - last_release_slot <= curr_slot
    fn any_ps_v2(
        SyncSolVal {
            lst_sol_val,
            curr_slot,
        }: SyncSolVal,
    ) -> impl Strategy<Value = PoolStateV2> {
        (
            *lst_sol_val.old()..=(u64::MAX - lst_sol_val.new()).saturating_add(*lst_sol_val.old()),
            0..=curr_slot,
        )
            .prop_map(|(total_sol_value, last_release_slot)| PoolStateV2 {
                total_sol_value,
                last_release_slot,
                ..Default::default()
            })
    }

    fn identical_sol_val_snap_strat() -> impl Strategy<Value = SnapU64> {
        any::<u64>().prop_map(|x| SnapU64::from_destr(SnapDestr { old: x, new: x }))
    }

    fn distinct_sol_val_snap_strat() -> impl Strategy<Value = SnapU64> {
        any::<u64>()
            .prop_flat_map(|old| {
                (
                    Just(old),
                    any::<u64>().prop_filter("must be distinct", move |x| *x != old),
                )
            })
            .prop_map(|(old, new)| SnapU64::from_destr(SnapDestr { old, new }))
    }

    proptest! {
        #[test]
        fn no_ssv_change_means_no_last_release_slot_update_pt(
            (mut ps, ssv) in identical_sol_val_snap_strat()
                .prop_flat_map(any_curr_slot_ssv_strat)
                .prop_flat_map(|ssv| (any_ps_v2(ssv), Just(ssv)))
        ) {
            let old_last_release_slot = ps.last_release_slot;
            ps.apply_ssv_uy(&ssv).unwrap();
            prop_assert_eq!(old_last_release_slot, ps.last_release_slot);
        }
    }

    proptest! {
        #[test]
        fn ssv_change_means_last_release_slot_update_pt(
            (mut ps, ssv) in distinct_sol_val_snap_strat()
                .prop_flat_map(any_curr_slot_ssv_strat)
                .prop_flat_map(|ssv| (any_ps_v2(ssv), Just(ssv)))
        ) {
            ps.apply_ssv_uy(&ssv).unwrap();
            prop_assert_eq!(ssv.curr_slot, ps.last_release_slot);
        }
    }
}
