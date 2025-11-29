use inf1_ctl_core::{
    typedefs::{pool_sv::PoolSvLamports, snap::SnapU64},
    yields::update::UpdateYield,
};

/// Sync SOL value of a single LST
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct SyncSolVal {
    /// Snapshot of lst_state sol value across time to determine change
    pub lst: SnapU64,
}

impl SyncSolVal {
    /// Returns new pool fields
    #[inline]
    pub const fn exec(self, pool: PoolSvLamports) -> Option<PoolSvLamports> {
        let Self { lst } = self;
        let sub_old = match pool.total().checked_sub(*lst.old()) {
            None => return None,
            Some(x) => x,
        };
        let new_total = match sub_old.checked_add(*lst.new()) {
            None => return None,
            Some(x) => x,
        };
        UpdateYield {
            new_total_sol_value: new_total,
            old: pool,
        }
        .exec()
    }

    // TODO: require variant with inf supply snap for Add/Remove liquidity

    // TODO: require variant without UpdateYield for the last sync in StartRebalance
}
