#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SyncSolVal {
    /// Current `pool.total_sol_value`
    pub pool_total: u64,

    /// LST's old reserves SOL value, probably read from `LstStateList`
    pub lst_old: u64,

    /// LST's new reserves SOL value, probably read from `sol_val_calc.lst_to_sol(balance)?.start()`
    pub lst_new: u64,
}

impl SyncSolVal {
    /// Returns new `pool.total_sol_value`
    #[inline]
    pub const fn exec(self) -> u64 {
        let Self {
            pool_total,
            lst_old,
            lst_new,
        } = self;
        pool_total - lst_old + lst_new
    }

    /// Returns new `pool.total_sol_value`
    #[inline]
    pub const fn exec_checked(self) -> Option<u64> {
        let Self {
            pool_total,
            lst_old,
            lst_new,
        } = self;
        match pool_total.checked_sub(lst_old) {
            None => None,
            Some(r) => r.checked_add(lst_new),
        }
    }
}
