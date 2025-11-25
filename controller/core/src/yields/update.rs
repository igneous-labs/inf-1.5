use generic_array_struct::generic_array_struct;

use crate::{accounts::pool_state::PoolStateV2, typedefs::snap::SnapU64};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UpdateYield {
    pub pool_total_sol_value: SnapU64,
    pub old_lamport_fields: YieldLamportFieldsVal,
}

impl UpdateYield {
    /// # Returns
    ///
    /// `None` on overflow on protocol fee calculation
    #[inline]
    pub const fn calc(&self) -> YieldLamportFieldUpdates {
        let (vals, dir) = if *self.pool_total_sol_value.old() <= *self.pool_total_sol_value.new() {
            // unchecked-arith: no overflow, bounds checked above
            let change = *self.pool_total_sol_value.new() - *self.pool_total_sol_value.old();
            (
                YieldLamportFieldsVal::memset(0).const_with_withheld(change),
                UpdateDir::Inc,
            )
        } else {
            // unchecked-arith: no overflow, bounds checked above
            let change = *self.pool_total_sol_value.old() - *self.pool_total_sol_value.new();
            let shortfall = change.saturating_sub(*self.old_lamport_fields.withheld());
            let withheld = change.saturating_sub(shortfall);
            (
                YieldLamportFieldsVal::memset(0)
                    .const_with_withheld(withheld)
                    .const_with_protocol_fee(shortfall),
                UpdateDir::Dec,
            )
        };
        YieldLamportFieldUpdates { vals, dir }
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
    /// `None` if increment overflows
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

    use inf1_test_utils::bals_from_supply;
    use proptest::prelude::*;

    use crate::{
        accounts::pool_state::{
            NewPoolStateV2U64sBuilder, PoolStateV2Addrs, PoolStateV2FTVals, PoolStateV2U8Bools,
        },
        typedefs::{
            fee_nanos::test_utils::any_fee_nanos_strat, rps::test_utils::any_rps_strat,
            snap::NewSnapBuilder,
        },
    };

    use super::*;

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
            prop_assert!(
                old_total_sol_value >= ps.protocol_fee_lamports + ps.withheld_lamports
            );

            let uy = UpdateYield {
                pool_total_sol_value: NewSnapBuilder::start()
                    .with_new(ps.total_sol_value)
                    .with_old(old_total_sol_value)
                    .build(),
                old_lamport_fields: NewYieldLamportFieldsBuilder::start()
                    .with_protocol_fee(ps.protocol_fee_lamports)
                    .with_withheld(ps.withheld_lamports)
                    .build(),
            };
            let u = uy.calc();

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
                None => {
                    match dir {
                        UpdateDir::Dec => panic!("decrement should never panic"),
                        UpdateDir::Inc => assert!(itr.any(|(old, c)| old.checked_add(c).is_none()))
                    }
                    // tests below assume update was successful
                    return Ok(());
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

            let [old_lp_sv, new_lp_sv] = [
                [*old_vals.protocol_fee(), *old_vals.withheld(), old_total_sol_value],
                [ps.protocol_fee_lamports, ps.withheld_lamports, ps.total_sol_value],
            ].map(|[p, w, t]| t - p - w);

            // sol value due to LPers should not change on profit events
            if let UpdateDir::Inc = dir {
                prop_assert_eq!(new_lp_sv, old_lp_sv, "{} != {}", old_lp_sv, new_lp_sv);
            }

            if let UpdateDir::Dec = dir {
                let [loss, lp_loss, withheld_loss, pf_loss] = [
                    [old_total_sol_value, ps.total_sol_value],
                    [old_lp_sv, new_lp_sv],
                    [*old_vals.withheld(), ps.withheld_lamports],
                    [*old_vals.protocol_fee(), ps.protocol_fee_lamports]
                ].map(|[o, n]| o - n);

                // sol value due to LPers should decrease by at most same amount on loss events.
                if *old_vals.withheld() + *old_vals.protocol_fee() == 0 {
                    // strict-eq if no softening
                    prop_assert_eq!(loss, lp_loss, "{} != {}", lp_loss, loss);

                } else {
                    // less if softened by accumulated withheld and protocol fees
                    prop_assert!(loss > lp_loss, "{} > {}", lp_loss, loss);
                }

                // accumulated withheld and protocol fee lamports should have in total
                // decreased at most equal to loss
                let non_lp_loss = withheld_loss + pf_loss;
                if ps.withheld_lamports + ps.protocol_fee_lamports == 0 {
                    prop_assert!(loss >= non_lp_loss, "{} > {}", non_lp_loss, loss);
                } else {
                    // strict-eq if no saturation
                    prop_assert_eq!(loss, non_lp_loss, "{} != {}", non_lp_loss, loss);
                }

                if pf_loss > 0 && *old_vals.withheld() > 0 {
                    prop_assert!(
                        withheld_loss > 0,
                        "withheld should be decreased from first before protocol fee"
                    );
                }
            }

            // after update_yield, total_sol_value must remain
            // >= protocol_fee_lamports + withheld_lamports,
            // assuming invariant holds before the update
            prop_assert!(
                ps.total_sol_value >= ps.protocol_fee_lamports + ps.withheld_lamports,
                "{} {} {}\n{} {} {}",
                old_vals.protocol_fee(), old_vals.withheld(), old_total_sol_value,
                ps.protocol_fee_lamports, ps.withheld_lamports, ps.total_sol_value,
            );
        }
    }
}
