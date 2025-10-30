/// Balance between large size to cover cases and small size for proptest exec speed
pub const MAX_LST_STATES: usize = 128;

/// To give us an upper bound on sol value of stake pools
/// that have exchange rate > 1
pub const MAX_LAMPORTS_OVER_SUPPLY: u64 = 1_000_000_000;

/// Calculate the maximum value that can be set without causing overflow
/// when updating pool sol value
pub const fn max_sol_val_no_overflow(pool_total_sol_val: u64, old_lst_state_sol_val: u64) -> u64 {
    u64::MAX - (pool_total_sol_val - old_lst_state_sol_val)
}
