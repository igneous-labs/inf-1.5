use crate::typedefs::{pool_sv::PoolSvLamports, update_dir::UpdateDir};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UpdateYield {
    pub new_total_sol_value: u64,
    pub old: PoolSvLamports,
}

impl UpdateYield {
    /// # Returns
    ///
    /// `None` on overflow on protocol fee calculation
    #[inline]
    pub const fn calc(&self) -> PoolSvUpdates {
        let (vals, dir) = if *self.old.total() <= self.new_total_sol_value {
            // unchecked-arith: no overflow, bounds checked above
            let change = self.new_total_sol_value - *self.old.total();
            (
                PoolSvLamports::memset(0)
                    .const_with_total(change)
                    .const_with_withheld(change),
                UpdateDir::Inc,
            )
        } else {
            // unchecked-arith: no overflow, bounds checked above
            let change = *self.old.total() - self.new_total_sol_value;
            let shortfall = change.saturating_sub(*self.old.withheld());
            let withheld = change.saturating_sub(shortfall);
            (
                PoolSvLamports::memset(0)
                    .const_with_total(change)
                    .const_with_withheld(withheld)
                    .const_with_protocol_fee(shortfall),
                UpdateDir::Dec,
            )
        };
        PoolSvUpdates { vals, dir }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PoolSvUpdates {
    pub vals: PoolSvLamports,
    pub dir: UpdateDir,
}

impl PoolSvUpdates {
    /// # Returns
    /// new values of `PoolValLamports`
    ///
    /// # Safety
    /// - Do not use onchain, UpdateDir::Inc can panic on overflow
    #[inline]
    pub fn exec(self, mut old: PoolSvLamports) -> PoolSvLamports {
        let Self { vals, dir } = self;
        vals.0.into_iter().zip(old.0.each_mut()).for_each(|(v, r)| {
            let new = match dir {
                UpdateDir::Dec => r.saturating_sub(v),
                UpdateDir::Inc => *r + v,
            };
            *r = new;
        });
        old
    }

    /// # Returns
    /// new values of `YieldLamportFieldsVal`
    ///
    /// `None` if increment overflows
    #[inline]
    pub fn exec_checked(self, mut old: PoolSvLamports) -> Option<PoolSvLamports> {
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
            NewPoolStateV2U64sBuilder, PoolStateV2, PoolStateV2Addrs, PoolStateV2FtaVals,
            PoolStateV2U8Bools,
        },
        typedefs::{
            fee_nanos::test_utils::any_ctl_fee_nanos_strat, pool_sv::PoolSvMutRefs,
            rps::test_utils::any_rps_strat,
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
                            new_tsv,
                            NewPoolStateV2U64sBuilder::start()
                                .with_last_release_slot(last_release_slot)
                                .with_protocol_fee_lamports(protocol_fee_lamports)
                                .with_total_sol_value(old_tsv)
                                .with_withheld_lamports(withheld_lamports)
                                .build(),
                        )
                    },
                ),
            array::from_fn(|_| any::<[u8; 32]>()),
            array::from_fn(|_| any::<u8>()),
            any_ctl_fee_nanos_strat(),
            any_rps_strat(),
        )
            .prop_map(
                |((new_tsv, u64s), addrs, u8_bools, protocol_fee_nanos, rps)| {
                    (
                        new_tsv,
                        PoolStateV2FtaVals {
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
            (new_total_sol_value, mut ps) in any_update_yield_strat(),
        ) {
            prop_assert!(
                ps.total_sol_value >= ps.protocol_fee_lamports + ps.withheld_lamports
            );

            let uy = UpdateYield {
                new_total_sol_value,
                old: PoolSvLamports::snap(&ps),
            };
            let u = uy.calc();

            let PoolSvUpdates { vals, dir } = u;
            let old_vals = uy.old;
            let exec_res = u.exec_checked(old_vals);
            let mut itr = old_vals.0.into_iter().zip(vals.0);
            let mut ps_refs = PoolSvMutRefs::from_pool_state_v2(&mut ps);

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
                    ps_refs.update(new_vals);
                    itr.zip(ps_refs.0).for_each(
                        |((old, u), ps_ref)| {
                            match dir {
                                UpdateDir::Inc => assert_eq!(*ps_ref, old + u),
                                UpdateDir::Dec => assert_eq!(*ps_ref, old.saturating_sub(u)),
                            }
                        }
                    );
                }
            }

            let new_vals = PoolSvLamports::snap(&ps);
            let [old_lp_sv, new_lp_sv] = [old_vals, new_vals]
                .each_ref()
                .map(PoolSvLamports::lp_due);

            // sol value due to LPers should not change on profit events
            if let UpdateDir::Inc = dir {
                prop_assert_eq!(new_lp_sv, old_lp_sv, "{} != {}", old_lp_sv, new_lp_sv);
            }

            if let UpdateDir::Dec = dir {
                let [loss, lp_loss, withheld_loss, pf_loss] = [
                    [*old_vals.total(), ps.total_sol_value],
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
                old_vals.protocol_fee(), old_vals.withheld(), new_total_sol_value,
                ps.protocol_fee_lamports, ps.withheld_lamports, ps.total_sol_value,
            );
        }
    }
}
