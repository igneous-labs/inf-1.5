use sanctum_u64_ratio::{Ceil, Ratio};

use crate::typedefs::{
    pool_sv::{PoolSv, PoolSvLamports},
    snap::SnapU64,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UpdateYield {
    pub new_total_sol_value: u64,
    pub old: PoolSvLamports,
}

/// Normalize old total sol value to new total sol value
/// by returning old * new_inf_supply / old_inf_supply
///
/// Edge cases
/// - if old_inf_supply = 0, then new_inf_supply is returned,
///   to be in-line with 1:1 exchange rate policy when INF supply is 0.
///   See [`crate::svc::InfCalc::sol_to_inf`]
/// - if new_inf_supply = 0, then 0 is returned, so all remaining SOL value
///   in the pool after all LPs have exited is treated as gains
#[inline]
pub const fn norm_old_total_sol_value(
    old_total_sol_value: u64,
    inf_supply: SnapU64,
) -> Option<u64> {
    let n = *inf_supply.new();
    let d = *inf_supply.old();
    if d == 0 {
        return Some(n);
    }
    if n == 0 {
        return Some(0);
    }
    Ceil(Ratio { n, d }).apply(old_total_sol_value)
}

impl UpdateYield {
    /// Normalize old total sol value to new total sol value
    /// by returning old * new_inf_supply / old_inf_supply
    ///
    /// Edge cases
    /// - if old_inf_supply = 0, then new_inf_supply is returned,
    ///   to be in-line with 1:1 exchange rate policy when INF supply is 0.
    ///   See [`crate::svc::InfCalc::sol_to_inf`]
    /// - if new_inf_supply = 0, then 0 is returned, so all remaining SOL value
    ///   in the pool after all LPs have exited is treated as gains
    #[inline]
    pub const fn normalized(self, inf_supply: SnapU64) -> Option<Self> {
        let old_total_sol_value = match norm_old_total_sol_value(*self.old.total(), inf_supply) {
            None => return None,
            Some(x) => x,
        };
        let Self {
            new_total_sol_value,
            old,
        } = self;
        Some(Self {
            new_total_sol_value,
            old: old.const_with_total(old_total_sol_value),
        })
    }

    /// # Returns
    /// New values of PoolSvLamports
    ///
    /// `None` on overflow
    #[inline]
    pub const fn exec(&self) -> Option<PoolSvLamports> {
        let [withheld, protocol_fee] = if self.new_total_sol_value >= *self.old.total() {
            // unchecked-arith: bounds checked above
            let norm_gains = self.new_total_sol_value - *self.old.total();
            [
                // saturation: can overflow if new_total_sol_value is large
                // and norm_old_total_sol_value < old.withheld. In this case,
                // rely on clamping below to ensure LP solvency invariant
                self.old.withheld().saturating_add(norm_gains),
                *self.old.protocol_fee(),
            ]
        } else {
            // unchecked-arith: bounds checked above
            let norm_losses = *self.old.total() - self.new_total_sol_value;
            let withheld_shortfall = norm_losses.saturating_sub(*self.old.withheld());
            [
                self.old.withheld().saturating_sub(norm_losses),
                self.old.protocol_fee().saturating_sub(withheld_shortfall),
            ]
        };

        // clamp withheld and protocol_fee to ensure LP solvency invariant
        let excess = (withheld as u128 + protocol_fee as u128)
            .saturating_sub(self.new_total_sol_value as u128);
        let excess = if excess > u64::MAX as u128 {
            u64::MAX
        } else {
            excess as u64
        };
        let withheld_shortfall = excess.saturating_sub(withheld);
        let withheld = withheld.saturating_sub(excess);
        let protocol_fee = protocol_fee.saturating_sub(withheld_shortfall);

        Some(
            PoolSv::memset(0)
                .const_with_total(self.new_total_sol_value)
                .const_with_withheld(withheld)
                .const_with_protocol_fee(protocol_fee),
        )
    }
}

#[cfg(test)]
mod tests {
    use inf1_test_utils::bals_from_supply;
    use proptest::prelude::*;

    use crate::typedefs::{pool_sv::NewPoolSvBuilder, snap::NewSnapBuilder};

    use super::*;

    /// Given old total sol value, gens changes in inf supply that will not overflow
    fn inf_supply_snap_strat(old_tsv: u64) -> impl Strategy<Value = SnapU64> {
        // n=new_inf_supply, d=old_inf_supply, M=u64::MAX
        // old * n / d <= M
        // old * n <= M * d
        // n <= M * d / old
        any::<u64>()
            .prop_map(move |d| {
                (
                    u128::from(u64::MAX) * u128::from(d) / u128::from(old_tsv),
                    d,
                )
            })
            .prop_flat_map(|(n_max, d)| {
                let n_max = if n_max > u128::from(u64::MAX) {
                    u64::MAX
                } else {
                    n_max.try_into().unwrap()
                };
                (0..=n_max, Just(d))
            })
            .prop_map(|(n, d)| NewSnapBuilder::start().with_new(n).with_old(d).build())
    }

    /// Gens PoolSvLamports where the invariant
    ///
    /// total_sol_value >= protocol_fee_lamports + withheld_lamports
    ///
    /// holds
    fn pool_sv_lamports_invar_strat(tsv: u64) -> impl Strategy<Value = PoolSvLamports> {
        bals_from_supply(tsv).prop_map(move |([withheld, protocol_fee], _rem)| {
            NewPoolSvBuilder::start()
                .with_protocol_fee(protocol_fee)
                .with_withheld(withheld)
                .with_total(tsv)
                .build()
        })
    }

    fn any_update_yield_strat() -> impl Strategy<Value = (UpdateYield, SnapU64)> {
        any::<u64>()
            .prop_flat_map(|old_tsv| {
                (
                    any::<u64>(),
                    pool_sv_lamports_invar_strat(old_tsv),
                    inf_supply_snap_strat(old_tsv),
                )
            })
            .prop_map(|(new_tsv, old, inf_supply)| {
                (
                    UpdateYield {
                        new_total_sol_value: new_tsv,
                        old,
                    },
                    inf_supply,
                )
            })
    }

    /// Tests that apply to all instances of UpdateYield
    fn uy_tests_all(uy: UpdateYield) {
        let old = uy.old;
        let new = uy.exec().unwrap();

        let [pf_loss, withheld_loss] = [
            [old.protocol_fee(), new.protocol_fee()],
            [old.withheld(), new.withheld()],
        ]
        .map(|[o, n]| o.saturating_sub(*n));

        if pf_loss > 0 && *old.withheld() > 0 {
            assert!(
                withheld_loss > 0,
                "withheld should be decreased from first before protocol fee"
            );
        }

        // LP solvent invariant
        assert!(*new.total() >= *new.protocol_fee() + *new.withheld());
        // TODO: add more props
    }

    proptest! {
        #[test]
        fn update_yield_pt(
            (uy, inf_supply) in any_update_yield_strat(),
        ) {
            // inf_supply_snap_strat should mean no overflow
            uy_tests_all(uy.normalized(inf_supply).unwrap());
        }
    }

    fn update_yield_inf_unchanged_strat() -> impl Strategy<Value = UpdateYield> {
        any::<u64>()
            .prop_flat_map(|old_tsv| (any::<u64>(), pool_sv_lamports_invar_strat(old_tsv)))
            .prop_map(|(new_total_sol_value, old)| UpdateYield {
                new_total_sol_value,
                old,
            })
    }

    fn uy_tests_inf_unchanged(uy: UpdateYield) {
        // TODO: let L = sol value due to LP, I = INF supply
        // enforce invariant that L_new / I_new = L_old / I_old for profit

        let old = uy.old;
        let new = uy.exec().unwrap();

        // this asserts the LP solvent invariant
        let [old_lp_sv, new_lp_sv] = [old, new].each_ref().map(PoolSvLamports::lp_due_checked);
        let old_lp_sv = old_lp_sv.unwrap();
        let new_lp_sv = new_lp_sv.ok_or((uy, new)).unwrap();

        if *new.total() >= *uy.old.total() {
            // profit event
            assert_eq!(
                new_lp_sv, old_lp_sv,
                "{} != {}. SOL value due to LPers should not change",
                old_lp_sv, new_lp_sv
            );
            assert_eq!(
                new.protocol_fee(),
                old.protocol_fee(),
                "{} != {}. protocol fee lamports should not change",
                new.protocol_fee(),
                old.protocol_fee(),
            );
            let profit = new.total() - *uy.old.total();
            let expected_withheld = old.withheld() + profit;
            assert_eq!(
                *new.withheld(),
                expected_withheld,
                "{} != {} + {}. withheld lamports don't match",
                new.withheld(),
                old.withheld(),
                profit,
            );
        } else {
            // loss event
            let loss = uy.old.total() - new.total();

            // sol value due to LPers should decrease by at most loss
            let lp_loss = old_lp_sv - new_lp_sv;
            assert!(lp_loss <= loss, "{} > {}", lp_loss, loss);

            // accumulated withheld and protocol fee lamports should have in total
            // decreased at most equal to loss
            let [pf_loss, withheld_loss] = [
                [old.protocol_fee(), new.protocol_fee()],
                [old.withheld(), new.withheld()],
            ]
            .map(|[o, n]| o - n);
            let non_lp_loss = withheld_loss + pf_loss;
            if old.withheld() + old.protocol_fee() < loss {
                assert!(loss > non_lp_loss, "{} > {}", non_lp_loss, loss);
            } else {
                // strict-eq if no saturation
                assert_eq!(loss, non_lp_loss);
            }
        }
    }

    proptest! {
        #[test]
        fn update_yield_inf_unchanged_pt(
            uy in update_yield_inf_unchanged_strat(),
        ) {
            uy_tests_all(uy);
            uy_tests_inf_unchanged(uy);
        }
    }
}
