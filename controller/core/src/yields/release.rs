//! The subsystem controlling the deferred release of yield over time

use generic_array_struct::generic_array_struct;
use sanctum_fee_ratio::BefFee;
use sanctum_u64_ratio::Ceil;

use crate::typedefs::{fee_nanos::FeeNanos, rps::Rps};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ReleaseYield {
    pub slots_elapsed: u64,
    pub withheld_lamports: u64,
    pub rps: Rps,
    pub protocol_fee_nanos: FeeNanos,
}

#[generic_array_struct(builder pub)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct YRel<T> {
    pub released: T,
    pub to_protocol: T,
    pub new_withheld: T,
}

impl<T: Copy> YRel<T> {
    #[inline]
    pub const fn memset(v: T) -> Self {
        Self([v; Y_REL_LEN])
    }
}

/// invariant: self.sum() = old_withheld_lamports
pub type YRelLamports = YRel<u64>;

impl ReleaseYield {
    /// # Returns
    ///
    /// If `new_withheld == old_withheld` then `pool_state.last_release_slot` should not be updated.
    ///
    /// Returns `None` on ratio error (overflow)
    #[inline]
    pub const fn calc(&self) -> YRelLamports {
        let Self {
            slots_elapsed,
            withheld_lamports,
            rps,
            protocol_fee_nanos,
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
        let bef_pf = BefFee(*withheld_lamports)
            .with_rem(new_withheld_lamports)
            .unwrap();
        let released_bef_pf = bef_pf.fee();
        // unwrap-safety: ratio.apply should never overflow since fee ratios <= 1.0
        let aft_pf = protocol_fee_nanos
            .into_fee()
            .apply(released_bef_pf)
            .unwrap();

        YRelLamports::memset(0)
            .const_with_new_withheld(new_withheld_lamports)
            .const_with_released(aft_pf.rem())
            .const_with_to_protocol(aft_pf.fee())
    }
}

#[cfg(test)]
mod tests {
    use expect_test::expect;
    use proptest::prelude::*;

    use crate::typedefs::{
        fee_nanos::test_utils::any_ctl_fee_nanos_strat, rps::test_utils::any_rps_strat,
        uq0f63::UQ0F63,
    };

    use super::*;

    fn into_ry(
        (slots_elapsed, withheld_lamports, rps, protocol_fee_nanos): (u64, u64, Rps, FeeNanos),
    ) -> ReleaseYield {
        ReleaseYield {
            slots_elapsed,
            withheld_lamports,
            rps,
            protocol_fee_nanos,
        }
    }

    fn any_release_yield_strat() -> impl Strategy<Value = ReleaseYield> {
        (
            any::<u64>(),
            any::<u64>(),
            any_rps_strat(),
            any_ctl_fee_nanos_strat(),
        )
            .prop_map(into_ry)
    }

    proptest! {
        #[test]
        fn release_yield_pt(ry in any_release_yield_strat()) {
            // shouldnt panic
            let res = ry.calc();

            // sum invariant
            prop_assert_eq!(
                res.0.into_iter().map(u128::from).sum::<u128>(),
                ry.withheld_lamports.into()
            );
        }
    }

    fn zero_slots_elapsed_strat() -> impl Strategy<Value = ReleaseYield> {
        (
            Just(0),
            any::<u64>(),
            any_rps_strat(),
            any_ctl_fee_nanos_strat(),
        )
            .prop_map(into_ry)
    }

    proptest! {
        #[test]
        fn zero_slots_elapsed_no_yields_released(ry in zero_slots_elapsed_strat()) {
            let ryc = ry.calc();
            prop_assert_eq!(*ryc.to_protocol(), 0);
            prop_assert_eq!(*ryc.released(), 0);
            prop_assert_eq!(*ryc.new_withheld(), ry.withheld_lamports);
        }
    }

    fn one_rps_strat() -> impl Strategy<Value = ReleaseYield> {
        (
            any::<u64>(),
            any::<u64>(),
            Just(Rps::new(UQ0F63::ONE).unwrap()),
            any_ctl_fee_nanos_strat(),
        )
            .prop_map(into_ry)
    }

    proptest! {
        #[test]
        fn one_rps_nonzero_slot_elapsed_release_all(ry in one_rps_strat()) {
            let res = ry.calc();
            match ry.slots_elapsed {
                0 => prop_assert_eq!(*res.new_withheld(), ry.withheld_lamports),
                _rest => prop_assert_eq!(*res.new_withheld(), 0)
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
                    Just(ry.protocol_fee_nanos),
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
            let aft_first = ry_sm.calc();
            let sm = ReleaseYield {
                slots_elapsed: ry_lg.slots_elapsed - ry_sm.slots_elapsed,
                withheld_lamports: *aft_first.new_withheld(),
                rps: ry_sm.rps,
                protocol_fee_nanos: ry_lg.protocol_fee_nanos,
            }.calc();
            prop_assert_eq!(lg.new_withheld(), sm.new_withheld());
            prop_assert_eq!(*lg.released(), aft_first.released() + sm.released());
            prop_assert_eq!(*lg.to_protocol(), aft_first.to_protocol() + sm.to_protocol());
        }
    }

    #[test]
    fn rand_rps_sc() {
        let ryc = ReleaseYield {
            slots_elapsed: 1,
            withheld_lamports: 2_000_000_000,
            // this is around 1 / 1_000_000_000
            rps: Rps::new(UQ0F63::new(9_223_372_037).unwrap()).unwrap(),
            protocol_fee_nanos: FeeNanos::new(1_000_000).unwrap(),
        }
        .calc();
        expect![[r#"
            YRel(
                [
                    1,
                    1,
                    1999999998,
                ],
            )
        "#]]
        .assert_debug_eq(&ryc);
    }
}
