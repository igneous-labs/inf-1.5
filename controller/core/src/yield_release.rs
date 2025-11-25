//! The subsystem controlling the deferred release of yield over time

use generic_array_struct::generic_array_struct;
use sanctum_fee_ratio::{AftFee, BefFee};
use sanctum_u64_ratio::Ceil;

use crate::typedefs::{fee_nanos::FeeNanos, rps::Rps, snap::SnapU64};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ReleaseYield {
    pub slots_elapsed: u64,
    pub withheld_lamports: u64,
    pub rps: Rps,
}

impl ReleaseYield {
    /// # Returns
    ///
    /// [`AftFee`] where
    /// - `.fee()` lamports to be released given slots_elapsed.
    ///   This can be 0 for small amounts of `slots_elapsed` and `rps`.
    ///   In those cases, `pool_state.last_release_slot` should not be updated.
    /// - `.rem()` new withheld lamports after release
    ///
    /// Returns `None` on ratio error (overflow)
    #[inline]
    pub const fn calc(&self) -> AftFee {
        let Self {
            slots_elapsed,
            withheld_lamports,
            rps,
        } = self;

        let rem_ratio = rps.as_inner().one_minus().pow(*slots_elapsed).into_ratio();
        let new_withheld_lamports = if rem_ratio.is_zero() {
            0
        } else {
            // use `Ceil` to round in favour of withholding more yield than necessary
            // unwrap-safety: .apply never panics because
            // - ratio > 0
            // - ratio <= 1, so never overflows
            Ceil(rem_ratio).apply(*withheld_lamports).unwrap()
        };
        // unwrap-safety: new_withheld_lamports is never > withheld_lamports
        // since its either 0 or * ratio where ratio <= 1.0
        BefFee(*withheld_lamports)
            .with_rem(new_withheld_lamports)
            .unwrap()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UpdateYield {
    pub pool_total_sol_value: SnapU64,
    pub protocol_fee_nanos: FeeNanos,
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

// dont derive Copy even tho we can. Same motivation as iterators
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct YieldLamportFieldUpdates {
    pub vals: YieldLamportFieldsVal,
    pub dir: UpdateDir,
}

impl UpdateYield {
    /// # Returns
    ///
    /// `None` on overflow
    #[inline]
    pub const fn calc(&self) -> Option<YieldLamportFieldUpdates> {
        let dir: UpdateDir;
        let vals: YieldLamportFieldsVal;

        if *self.pool_total_sol_value.old() <= *self.pool_total_sol_value.new() {
            dir = UpdateDir::Inc;
            // unchecked-arith: no overflow, bounds checked above
            let change = *self.pool_total_sol_value.new() - *self.pool_total_sol_value.old();
            let aft_pf = match self.protocol_fee_nanos.into_fee().apply(change) {
                None => return None,
                Some(a) => a,
            };
            vals = YieldLamportFieldsVal::memset(0)
                .const_with_protocol_fee(aft_pf.fee())
                .const_with_withheld(aft_pf.rem());
        } else {
            dir = UpdateDir::Dec;
            // unchecked-arith: no overflow, bounds checked above
            let change = *self.pool_total_sol_value.old() - *self.pool_total_sol_value.new();
            vals = YieldLamportFieldsVal::memset(0).const_with_withheld(change);
        }

        Some(YieldLamportFieldUpdates { vals, dir })
    }
}

pub type YieldLamportFieldsMut<'a> = YieldLamportFields<&'a mut u64>;

impl YieldLamportFieldUpdates {
    /// Consumes `self`
    ///
    /// # Returns
    /// `None` on overflow
    #[inline]
    pub fn exec(self, fields: YieldLamportFieldsMut) -> Option<()> {
        let Self { vals, dir } = self;
        vals.0.into_iter().zip(fields.0).try_for_each(|(v, r)| {
            let new = match dir {
                UpdateDir::Dec => r.checked_sub(v)?,
                UpdateDir::Inc => r.checked_add(v)?,
            };
            *r = new;
            Some(())
        })
    }
}

#[cfg(test)]
mod tests {
    use expect_test::expect;
    use proptest::prelude::*;

    use crate::typedefs::{rps::test_utils::any_rps_strat, uq0_63::UQ0_63};

    use super::*;

    fn into_ry((slots_elapsed, withheld_lamports, rps): (u64, u64, Rps)) -> ReleaseYield {
        ReleaseYield {
            slots_elapsed,
            withheld_lamports,
            rps,
        }
    }

    fn any_release_yield_strat() -> impl Strategy<Value = ReleaseYield> {
        (any::<u64>(), any::<u64>(), any_rps_strat()).prop_map(into_ry)
    }

    proptest! {
        #[test]
        fn release_yield_pt(ry in any_release_yield_strat()) {
            // sanctum-fee-ratio tests guarantee many props e.g.
            // - new_withheld_lamports + .fee() = withheld_lamports
            // - .fee() (released yield) <= withheld_lamports
            // - .rem() (new_withheld_lamports) <= withheld_lamports
            //
            // So just test that calc() never panics for all cases here
            ry.calc();
        }
    }

    fn zero_slots_elapsed_strat() -> impl Strategy<Value = ReleaseYield> {
        (Just(0), any::<u64>(), any_rps_strat()).prop_map(into_ry)
    }

    proptest! {
        #[test]
        fn zero_slots_elapsed_no_yields_released(ry in zero_slots_elapsed_strat()) {
            prop_assert_eq!(ry.calc().fee(), 0);
        }
    }

    fn one_rps_strat() -> impl Strategy<Value = ReleaseYield> {
        (
            any::<u64>(),
            any::<u64>(),
            Just(Rps::new(UQ0_63::ONE).unwrap()),
        )
            .prop_map(into_ry)
    }

    proptest! {
        #[test]
        fn one_rps_nonzero_slot_elapsed_release_all(ry in one_rps_strat()) {
            let res = ry.calc();
            match ry.slots_elapsed {
                // sanctum-fee-ratio guarantees .rem() == .bef_fee()
                // (new_withheld = withheld)
                0 => prop_assert_eq!(res.fee(), 0),

                // sanctum-fee-ratio guarantees .fee() == .bef_fee()
                // (released = withheld)
                _rest => prop_assert_eq!(res.rem(), 0)
            };
        }
    }

    /// Returns (release_yield, release_yield with same params but slots_elapsed <= .0's)
    fn release_yield_split_strat() -> impl Strategy<Value = (ReleaseYield, ReleaseYield)> {
        any_release_yield_strat().prop_flat_map(|ry| {
            (
                Just(ry),
                (
                    0..=ry.slots_elapsed,
                    Just(ry.withheld_lamports),
                    Just(ry.rps),
                )
                    .prop_map(into_ry),
            )
        })
    }

    proptest! {
        #[test]
        fn two_release_yields_in_seq_same_as_one_big_one(
            (ry_lg, ry_sm) in release_yield_split_strat()
        ) {
            let lg = ry_lg.calc();
            let sm = ReleaseYield {
                slots_elapsed: ry_lg.slots_elapsed - ry_sm.slots_elapsed,
                withheld_lamports: ry_sm.calc().rem(),
                rps: ry_sm.rps,
            }.calc();
            prop_assert_eq!(lg.rem(), sm.rem());
        }
    }

    #[test]
    fn rand_rps_sc() {
        let ryc = ReleaseYield {
            slots_elapsed: 1,
            withheld_lamports: 1_000_000_000,
            // this is around 1 / 1_000_000_000
            rps: Rps::new(UQ0_63::new(9_223_372_037).unwrap()).unwrap(),
        }
        .calc();
        let _ = [
            (
                expect![[r#"
                    999999999
                "#]],
                ryc.rem(),
            ),
            (
                expect![[r#"
                    1
                "#]],
                ryc.fee(),
            ),
            (
                expect![[r#"
                1000000000
            "#]],
                ryc.bef_fee(),
            ),
        ]
        .map(|(e, a)| e.assert_debug_eq(&a));
    }
}
