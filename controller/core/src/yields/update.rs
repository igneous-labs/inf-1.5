use generic_array_struct::generic_array_struct;

use crate::typedefs::{fee_nanos::FeeNanos, snap::SnapU64};

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
