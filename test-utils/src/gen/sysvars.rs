use core::array;

use generic_array_struct::generic_array_struct;
use proptest::prelude::*;
use solana_clock::Clock;

#[generic_array_struct]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClockU64s<T> {
    pub slot: T,
    pub epoch: T,
    pub leader_schedule_epoch: T,
}

#[generic_array_struct]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClockI64s<T> {
    pub unix_timestamp: T,
    pub epoch_start_timestamp: T,
}

pub const fn clock_from_fta(u64s: ClockU64s<u64>, i64s: ClockI64s<i64>) -> Clock {
    Clock {
        slot: *u64s.slot(),
        epoch_start_timestamp: *i64s.epoch_start_timestamp(),
        epoch: *u64s.epoch(),
        leader_schedule_epoch: *u64s.leader_schedule_epoch(),
        unix_timestamp: *i64s.unix_timestamp(),
    }
}

pub fn any_clock_strat() -> impl Strategy<Value = Clock> {
    (
        array::from_fn(|_| any::<u64>()),
        array::from_fn(|_| any::<i64>()),
    )
        .prop_map(|(u, i)| clock_from_fta(ClockU64s(u), ClockI64s(i)))
}
