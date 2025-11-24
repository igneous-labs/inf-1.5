//! The subsystem controlling the deferred release of yield over time

use sanctum_fee_ratio::{AftFee, BefFee};
use sanctum_u64_ratio::Ceil;

use crate::typedefs::rps::Rps;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ReleaseYield {
    pub slots_elapsed: u64,
    pub withheld_lamports: u64,
    pub rps: Rps,
}

impl ReleaseYield {
    /// # Returns
    /// - `.fee()` lamports to be released given slots_elapsed
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

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use crate::typedefs::{rps::test_utils::any_rps_strat, uq0_64::UQ0_64};

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

    fn one_rps_strat() -> impl Strategy<Value = ReleaseYield> {
        (
            any::<u64>(),
            any::<u64>(),
            Just(Rps::new(UQ0_64::ONE).unwrap()),
        )
            .prop_map(into_ry)
    }

    proptest! {
        #[test]
        fn release_yield_pt(ry in any_release_yield_strat()) {
            // sanctum-fee-ratio tests guarantee many props e.g.
            // - new_withheld_lamports + .fee() = withheld_lamports
            //
            // So just test that calc() never panics for all cases here
            ry.calc();
        }
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
}
