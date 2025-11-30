use inf1_ctl_core::typedefs::snap::SnapU64;

// This is in top-level core as a general useful utility both onchain and offchain;
// used offchain to perform manual syncs in case of stale SOL values

/// Sync SOL value of a single LST
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct SyncSolVal {
    /// Snapshot of lst_state sol value across time to determine change
    pub lst_sol_val: SnapU64,
}

impl SyncSolVal {
    /// Returns new pool total SOL value
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
