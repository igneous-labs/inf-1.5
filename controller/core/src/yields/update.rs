use generic_array_struct::generic_array_struct;

use crate::{
    accounts::pool_state::PoolStateV2,
    typedefs::{
        fee_nanos::{FeeNanos, FeeNanosTooLargeErr},
        snap::SnapU64,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UpdateYield {
    pub pool_total_sol_value: SnapU64,
    pub protocol_fee_nanos: FeeNanos,
}

impl UpdateYield {
    #[inline]
    pub const fn new(
        old_total_sol_value: u64,
        PoolStateV2 {
            protocol_fee_nanos,
            total_sol_value,
            ..
        }: &PoolStateV2,
    ) -> Result<Self, FeeNanosTooLargeErr> {
        let protocol_fee_nanos = match FeeNanos::new(*protocol_fee_nanos) {
            Err(e) => return Err(e),
            Ok(v) => v,
        };
        Ok(Self {
            protocol_fee_nanos,
            pool_total_sol_value: SnapU64::memset(0)
                .const_with_new(*total_sol_value)
                .const_with_old(old_total_sol_value),
        })
    }

    /// # Returns
    ///
    /// `None` on overflow on protocol fee calculation
    #[inline]
    pub const fn calc(&self) -> Option<YieldLamportFieldUpdates> {
        let (change, dir) = if *self.pool_total_sol_value.old() <= *self.pool_total_sol_value.new()
        {
            // unchecked-arith: no overflow, bounds checked above
            (
                *self.pool_total_sol_value.new() - *self.pool_total_sol_value.old(),
                UpdateDir::Inc,
            )
        } else {
            // unchecked-arith: no overflow, bounds checked above
            (
                *self.pool_total_sol_value.old() - *self.pool_total_sol_value.new(),
                UpdateDir::Dec,
            )
        };
        let aft_pf = match self.protocol_fee_nanos.into_fee().apply(change) {
            None => return None,
            Some(a) => a,
        };

        Some(YieldLamportFieldUpdates {
            vals: YieldLamportFieldsVal::memset(0)
                .const_with_protocol_fee(aft_pf.fee())
                .const_with_withheld(aft_pf.rem()),
            dir,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UpdateDir {
    /// increment
    Inc,

    /// decrement
    Dec,
}

#[generic_array_struct(builder pub)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct YieldLamportFields<T> {
    /// `pool_state.withheld_lamports`
    pub withheld: T,

    /// `pool_state.protocol_fee_lamports`
    pub protocol_fee: T,
}

impl<T: Copy> YieldLamportFields<T> {
    #[inline]
    pub const fn memset(v: T) -> Self {
        Self([v; YIELD_LAMPORT_FIELDS_LEN])
    }
}

pub type YieldLamportFieldsVal = YieldLamportFields<u64>;

impl YieldLamportFieldsVal {
    #[inline]
    pub const fn snap(
        PoolStateV2 {
            withheld_lamports,
            protocol_fee_lamports,
            ..
        }: &PoolStateV2,
    ) -> Self {
        YieldLamportFieldsVal::memset(0)
            .const_with_protocol_fee(*protocol_fee_lamports)
            .const_with_withheld(*withheld_lamports)
    }
}

// dont derive Copy even tho we can. Same motivation as iterators
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct YieldLamportFieldUpdates {
    pub vals: YieldLamportFieldsVal,
    pub dir: UpdateDir,
}

impl YieldLamportFieldUpdates {
    /// Consumes `self`
    ///
    /// # Returns
    /// new values of `YieldLamportFieldsVal`
    ///
    /// `None` on overflow
    #[inline]
    pub fn exec(self, mut old: YieldLamportFieldsVal) -> Option<YieldLamportFieldsVal> {
        let Self { vals, dir } = self;
        vals.0
            .into_iter()
            .zip(old.0.each_mut())
            .try_for_each(|(v, r)| {
                let new = match dir {
                    UpdateDir::Dec => r.saturating_sub(v),
                    UpdateDir::Inc => r.checked_add(v)?,
                };
                *r = new;
                Some(())
            })?;
        Some(old)
    }
}

#[cfg(test)]
mod tests {
    use core::array;

    use proptest::prelude::*;

    use crate::{
        accounts::pool_state::{
            NewPoolStateV2U64sBuilder, PoolStateV2Addrs, PoolStateV2FTVals, PoolStateV2U8Bools,
        },
        typedefs::{fee_nanos::test_utils::any_fee_nanos_strat, rps::test_utils::any_rps_strat},
    };

    use super::*;

    // below 2 fns copy-pastad from test-utils to avoid circ dep

    fn bals_from_supply<const N: usize>(supply: u64) -> impl Strategy<Value = ([u64; N], u64)> {
        let end = array::from_fn(|_| Just(0u64));
        (0..N).fold((end, Just(supply)).boxed(), |tup, i| {
            tup.prop_flat_map(|(end, rem)| (bal_from_supply(rem), Just(end)))
                .prop_map(move |((bal, rem), mut end)| {
                    end[i] = bal;
                    (end, rem)
                })
                .boxed()
        })
    }

    fn bal_from_supply(supply: u64) -> impl Strategy<Value = (u64, u64)> {
        (0..=supply).prop_map(move |bal| (bal, supply - bal))
    }

    /// Specifically for this subsystem only.
    ///
    /// total_sol_value should be the new sol value
    fn any_update_yield_strat() -> impl Strategy<Value = (u64, PoolStateV2)> {
        (
            any::<u64>()
                // this enforces the invariant that
                // old_total_sol_value >= old_protocol_fee_lamports + old_withheld_lamports
                .prop_flat_map(|old_tsv| {
                    ([any::<u64>(); 2], bals_from_supply(old_tsv), Just(old_tsv))
                })
                .prop_map(
                    |(
                        [new_tsv, last_release_slot],
                        ([withheld_lamports, protocol_fee_lamports], _rem),
                        old_tsv,
                    )| {
                        (
                            old_tsv,
                            NewPoolStateV2U64sBuilder::start()
                                .with_last_release_slot(last_release_slot)
                                .with_protocol_fee_lamports(protocol_fee_lamports)
                                .with_total_sol_value(new_tsv)
                                .with_withheld_lamports(withheld_lamports)
                                .build(),
                        )
                    },
                ),
            array::from_fn(|_| any::<[u8; 32]>()),
            array::from_fn(|_| any::<u8>()),
            any_fee_nanos_strat(),
            any_rps_strat(),
        )
            .prop_map(
                |((old_tsv, u64s), addrs, u8_bools, protocol_fee_nanos, rps)| {
                    (
                        old_tsv,
                        PoolStateV2FTVals {
                            addrs: PoolStateV2Addrs(addrs),
                            u64s,
                            u8_bools: PoolStateV2U8Bools(u8_bools),
                            protocol_fee_nanos,
                            rps,
                        }
                        .into_pool_state_v2(),
                    )
                },
            )
    }

    proptest! {
        #[test]
        fn update_yield_pt(
            (old_total_sol_value, mut ps) in any_update_yield_strat(),
        ) {
            prop_assert!(old_total_sol_value >= ps.protocol_fee_lamports + ps.withheld_lamports);

            let uy = UpdateYield::new(old_total_sol_value, &ps).unwrap();
            let u = uy.calc().unwrap();

            let YieldLamportFieldUpdates { vals, dir } = u;

            let old_vals = NewYieldLamportFieldsBuilder::start()
                .with_withheld(ps.withheld_lamports)
                .with_protocol_fee(ps.protocol_fee_lamports)
                .build();

            let exec_res = u.exec(YieldLamportFieldsVal::snap(&ps));

            let mut itr = old_vals.0.into_iter().zip(vals.0);

            let PoolStateV2 { withheld_lamports, protocol_fee_lamports, .. } = &mut ps;

            let ps_refs = NewYieldLamportFieldsBuilder::start()
                .with_withheld(withheld_lamports)
                .with_protocol_fee(protocol_fee_lamports)
                .build();

            match exec_res {
                None => match dir {
                    UpdateDir::Dec => panic!("decrement should never panic"),
                    UpdateDir::Inc => assert!(itr.any(|(old, c)| old.checked_add(c).is_none()))
                }
                Some(new_vals) => {
                    itr.zip(new_vals.0).zip(ps_refs.0).for_each(
                        |(((old, c), new), ps_ref)| {
                            match dir {
                                UpdateDir::Inc => assert_eq!(new, old + c),
                                UpdateDir::Dec => assert_eq!(new, old.saturating_sub(c)),
                            }
                            *ps_ref = new;
                        }
                    );
                }
            }

            // invariant: after update_yield, total_sol_value must remain
            // >= protocol_fee_lamports + withheld_lamports,
            // assuming invariant holds before the update
            // prop_assert!(
            //     ps.protocol_fee_lamports.checked_add(ps.withheld_lamports).is_some(),
            //     "{} {} {}\n{} {} {}",
            //     old_vals.protocol_fee(), old_vals.withheld(), old_total_sol_value,
            //     ps.protocol_fee_lamports, ps.withheld_lamports, ps.total_sol_value,
            // );
            prop_assert!(
                ps.total_sol_value >= ps.protocol_fee_lamports + ps.withheld_lamports,
                "{} {} {}\n{} {} {}",
                old_vals.protocol_fee(), old_vals.withheld(), old_total_sol_value,
                ps.protocol_fee_lamports, ps.withheld_lamports, ps.total_sol_value,
            );
        }
    }
}
