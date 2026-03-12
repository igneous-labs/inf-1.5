//! The subsystem controlling the deferred release of yield over time

use generic_array_struct::generic_array_struct;
use sanctum_fee_ratio::BefFee;
use sanctum_u64_ratio::Ceil;

use crate::{
    accounts::pool_state::PoolStateV2,
    err::{Inf1CtlErr, InvalidPoolStateDataErrV2},
    internal_utils::impl_gas_memset,
    typedefs::{fee_nanos::FeeNanos, pool_sv::PoolSvMutRefs, rps::Rps},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ReleaseYieldParams {
    pub slots_elapsed: u64,
    pub rps: Rps,
    pub protocol_fee_nanos: FeeNanos,
}

impl ReleaseYieldParams {
    #[inline]
    pub const fn new(ps: &PoolStateV2, curr_slot: u64) -> Result<Self, Inf1CtlErr> {
        Ok(Self {
            slots_elapsed: match curr_slot.checked_sub(ps.last_release_slot) {
                None => return Err(Inf1CtlErr::TimeWentBackwards),
                Some(x) => x,
            },
            rps: match ps.rps_checked() {
                Err(e) => {
                    return Err(Inf1CtlErr::InvalidPoolStateDataV2(
                        InvalidPoolStateDataErrV2::Rps(e),
                    ))
                }
                Ok(x) => x,
            },
            protocol_fee_nanos: match ps.protocol_fee_nanos_checked() {
                Err(e) => {
                    return Err(Inf1CtlErr::InvalidPoolStateDataV2(
                        InvalidPoolStateDataErrV2::ProtocolFeeNanos(e),
                    ))
                }
                Ok(x) => x,
            },
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ReleaseYield {
    pub params: ReleaseYieldParams,
    pub withheld_lamports: u64,
}

#[generic_array_struct(builder pub)]
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct YRel<T> {
    /// Excludes protocol fee's share of the released yield
    pub released: T,

    /// Protocol fee's share of the released yield
    pub to_protocol: T,

    /// New value of withheld_lamports remaining
    pub new_withheld: T,
}
impl_gas_memset!(YRel, Y_REL_LEN);

/// invariant: self.sum() = old_withheld_lamports
pub type YRelLamports = YRel<u64>;

impl ReleaseYield {
    #[inline]
    pub const fn new(ps: &PoolStateV2, curr_slot: u64) -> Result<Self, Inf1CtlErr> {
        Ok(Self {
            params: match ReleaseYieldParams::new(ps, curr_slot) {
                Err(e) => return Err(e),
                Ok(p) => p,
            },
            withheld_lamports: ps.withheld_lamports,
        })
    }

    /// # Returns
    ///
    /// If `new_withheld == old_withheld` then `pool_state.last_release_slot` should not be updated.
    #[inline]
    pub const fn calc(&self) -> YRelLamports {
        let Self {
            withheld_lamports,
            params:
                ReleaseYieldParams {
                    rps,
                    protocol_fee_nanos,
                    slots_elapsed,
                },
        } = self;

        let rem_ratio = rps.as_inner().one_minus().pow(*slots_elapsed).into_ratio();

        // use `Ceil` to round in favour of withholding more yield than necessary
        //
        // unwrap-safety: .apply never panics because
        // - ratio <= 1, so never overflows
        let new_withheld_lamports = Ceil(rem_ratio).apply(*withheld_lamports).unwrap();

        // unwrap-safety: new_withheld_lamports is never > withheld_lamports
        // since its * ratio where ratio <= 1.0
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

impl PoolSvMutRefs<'_> {
    /// # Returns
    /// `None` on overflow of protocol_fee_lamports
    #[inline]
    pub const fn apply_yrel(&mut self, yrel: YRelLamports) -> Option<&mut Self> {
        let new_pf_lamports = match self.protocol_fee().checked_add(*yrel.to_protocol()) {
            None => return None,
            Some(x) => x,
        };

        **self.protocol_fee_mut() = new_pf_lamports;
        **self.withheld_mut() = *yrel.new_withheld();
        // total unchanged

        Some(self)
    }
}

impl PoolStateV2 {
    /// # Returns
    /// `None` on overflow of protocol_fee_lamports
    #[inline]
    pub fn apply_yrel(&mut self, yrel: YRelLamports, curr_slot: u64) -> Option<&mut Self> {
        // only update last_release_slot on nonzero release
        let should_update_last_release_slot = self.withheld_lamports != *yrel.new_withheld();

        PoolSvMutRefs::from_pool_state_v2(self).apply_yrel(yrel)?;

        // update last_release_slot after fallible PoolSv so that
        // changes to these fields are atomic
        if should_update_last_release_slot {
            self.last_release_slot = curr_slot;
        }

        Some(self)
    }

    /// # Returns
    /// The yield release event
    #[inline]
    pub fn release_yield(&mut self, curr_slot: u64) -> Result<YRelLamports, Inf1CtlErr> {
        let yrel = ReleaseYield::new(self, curr_slot)?.calc();
        self.apply_yrel(yrel, curr_slot)
            .ok_or(Inf1CtlErr::MathError)?;
        Ok(yrel)
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
            params: ReleaseYieldParams {
                slots_elapsed,
                rps,
                protocol_fee_nanos,
            },
            withheld_lamports,
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
            match ry.params.slots_elapsed {
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
                    0..=ry.params.slots_elapsed,
                    Just(ry.withheld_lamports),
                    Just(ry.params.rps),
                    Just(ry.params.protocol_fee_nanos),
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
                params: ReleaseYieldParams {
                    slots_elapsed: ry_lg.params.slots_elapsed - ry_sm.params.slots_elapsed,
                    rps: ry_sm.params.rps,
                    protocol_fee_nanos: ry_lg.params.protocol_fee_nanos,
                },
                withheld_lamports: *aft_first.new_withheld(),
            }.calc();
            prop_assert_eq!(lg.new_withheld(), sm.new_withheld());
            prop_assert_eq!(*lg.released(), aft_first.released() + sm.released());
            prop_assert_eq!(*lg.to_protocol(), aft_first.to_protocol() + sm.to_protocol());
        }
    }

    #[test]
    fn rand_rps_sc() {
        let ryc = ReleaseYield {
            withheld_lamports: 2_000_000_000,
            params: ReleaseYieldParams {
                slots_elapsed: 1,
                protocol_fee_nanos: FeeNanos::new(1_000_000).unwrap(),
                // this is around 1 / 1_000_000_000
                rps: Rps::new(UQ0F63::new(9_223_372_037).unwrap()).unwrap(),
            },
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
