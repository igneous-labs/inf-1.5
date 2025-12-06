use generic_array_struct::generic_array_struct;

use crate::{accounts::pool_state::PoolStateV2, internal_utils::impl_gas_memset};

/// Pool SOL Values
#[generic_array_struct(builder pub)]
#[repr(transparent)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PoolSv<T> {
    pub total: T,
    pub withheld: T,
    pub protocol_fee: T,
}
impl_gas_memset!(PoolSv, POOL_SV_LEN);

pub type PoolSvLamports = PoolSv<u64>;

impl PoolSvLamports {
    #[inline]
    pub const fn from_pool_state_v2(
        PoolStateV2 {
            total_sol_value,
            withheld_lamports,
            protocol_fee_lamports,
            ..
        }: &PoolStateV2,
    ) -> Self {
        Self::memset(0)
            .const_with_total(*total_sol_value)
            .const_with_protocol_fee(*protocol_fee_lamports)
            .const_with_withheld(*withheld_lamports)
    }

    #[inline]
    pub const fn non_lp_checked(&self) -> Option<u64> {
        self.withheld().checked_add(*self.protocol_fee())
    }

    #[inline]
    pub const fn lp_due_checked(&self) -> Option<u64> {
        match self.non_lp_checked() {
            None => None,
            Some(x) => self.total().checked_sub(x),
        }
    }
}

pub type PoolSvMutRefs<'a> = PoolSv<&'a mut u64>;

impl<'a> PoolSvMutRefs<'a> {
    #[inline]
    pub fn from_pool_state_v2(
        PoolStateV2 {
            total_sol_value,
            protocol_fee_lamports,
            withheld_lamports,
            ..
        }: &'a mut PoolStateV2,
    ) -> Self {
        NewPoolSvBuilder::start()
            .with_protocol_fee(protocol_fee_lamports)
            .with_total(total_sol_value)
            .with_withheld(withheld_lamports)
            .build()
    }
}

impl PoolSvMutRefs<'_> {
    /// Set `self` to `vals`
    #[inline]
    pub fn update(&mut self, vals: PoolSvLamports) {
        self.0.iter_mut().zip(vals.0).for_each(|(r, x)| **r = x);
    }
}

#[cfg(test)]
pub mod test_utils {
    use inf1_test_utils::bals_from_supply;
    use proptest::prelude::*;

    use super::*;

    /// Gens PoolSvLamports where the invariant
    ///
    /// total_sol_value >= protocol_fee_lamports + withheld_lamports
    ///
    /// holds
    pub fn pool_sv_lamports_invar_strat(tsv: u64) -> impl Strategy<Value = PoolSvLamports> {
        bals_from_supply(tsv).prop_map(move |([withheld, protocol_fee], _rem)| {
            NewPoolSvBuilder::start()
                .with_protocol_fee(protocol_fee)
                .with_withheld(withheld)
                .with_total(tsv)
                .build()
        })
    }
}
