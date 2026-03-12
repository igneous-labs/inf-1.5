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
#[repr(transparent)]
pub struct SyncSolVal {
    /// Snapshot of lst_state sol value across time to determine change
    pub lst_sol_val: SnapU64,
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
        let Self { lst_sol_val } = self;
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
        Some(self)
    }
}
