use inf1_ctl_core::{
    typedefs::{pool_sv::PoolSvLamports, snap::SnapU64},
    yields::update::{PoolSvUpdates, UpdateYield},
};

/// Sync SOL value of a single LST
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct SyncSolVal {
    /// Snapshot of lst_state sol value across time to determine change
    pub lst: SnapU64,
}

impl SyncSolVal {
    /// Returns (new pool fields, updates performed)
    ///
    /// # Safety
    /// Do not use onchain, can panic on overflow
    #[inline]
    pub fn exec(self, pool: PoolSvLamports) -> (PoolSvLamports, PoolSvUpdates) {
        let Self { lst } = self;
        let new_total = pool.total() - lst.old() + lst.new();
        let updates = UpdateYield {
            new_total_sol_value: new_total,
            old: pool,
        }
        .calc();
        (updates.exec(pool), updates)
    }

    /// Returns (new pool fields, updates performed)
    #[inline]
    pub fn exec_checked(self, pool: PoolSvLamports) -> Option<(PoolSvLamports, PoolSvUpdates)> {
        let Self { lst } = self;
        let new_total = pool
            .total()
            .checked_sub(*lst.old())
            .and_then(|x| x.checked_add(*lst.new()))?;
        let updates = UpdateYield {
            new_total_sol_value: new_total,
            old: pool,
        }
        .calc();
        Some((updates.exec(pool), updates))
    }
}
