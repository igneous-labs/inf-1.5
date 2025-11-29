use inf1_ctl_core::{
    typedefs::{pool_sv::PoolSvLamports, snap::SnapU64},
    yields::update::UpdateYield,
};

/// Sync SOL value of a single LST
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SyncSolVal {
    /// Snapshot of lst_state sol value across time to determine change
    pub lst: SnapU64,

    pub inf_supply: SnapU64,
}

impl SyncSolVal {
    #[inline]
    pub const fn inf_supply_unchanged(lst: SnapU64) -> Self {
        Self {
            lst,
            inf_supply: SnapU64::memset(1),
        }
    }

    /// Returns new pool fields
    #[inline]
    pub const fn exec_checked(self, pool: PoolSvLamports) -> Option<PoolSvLamports> {
        let Self { lst, inf_supply } = self;
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
            inf_supply,
        }
        .exec()
    }

    // TODO: require variant without UpdateYield for the last sync in StartRebalance
}
