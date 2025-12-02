use crate::typedefs::pool_sv::{PoolSv, PoolSvLamports};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UpdateYield {
    pub new_total_sol_value: u64,
    pub old: PoolSvLamports,
}

impl UpdateYield {
    /// # Returns
    /// New values of PoolSvLamports
    ///
    /// `None` on overflow
    #[inline]
    pub const fn exec(&self) -> Option<PoolSvLamports> {
        let [withheld, protocol_fee] = if self.new_total_sol_value >= *self.old.total() {
            // unchecked-arith: bounds checked above
            let gains = self.new_total_sol_value - *self.old.total();
            [
                // saturation: can overflow if new_total_sol_value is large
                // and norm_old_total_sol_value < old.withheld. In this case,
                // rely on clamping below to ensure LP solvency invariant
                self.old.withheld().saturating_add(gains),
                *self.old.protocol_fee(),
            ]
        } else {
            // unchecked-arith: bounds checked above
            let losses = *self.old.total() - self.new_total_sol_value;
            let withheld_shortfall = losses.saturating_sub(*self.old.withheld());
            [
                self.old.withheld().saturating_sub(losses),
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

    use crate::typedefs::pool_sv::NewPoolSvBuilder;

    use super::*;

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

    fn any_update_yield_strat() -> impl Strategy<Value = UpdateYield> {
        any::<u64>()
            .prop_flat_map(|old_tsv| (any::<u64>(), pool_sv_lamports_invar_strat(old_tsv)))
            .prop_map(|(new_total_sol_value, old)| UpdateYield {
                new_total_sol_value,
                old,
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

        // lp_due_checked asserts the LP solvent invariant
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
            uy in any_update_yield_strat(),
        ) {
            uy_tests_all(uy);
        }
    }
}
