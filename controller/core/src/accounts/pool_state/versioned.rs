use crate::{
    accounts::pool_state::{PoolState, PoolStatePacked, PoolStateV2, PoolStateV2Packed},
    typedefs::{fee_nanos::NANOS_DENOM, rps::Rps, versioned::V1_2},
    v1_2_each_field, v1_2_each_field_mut, v1_2_each_meth,
};

pub type VerPoolState = V1_2<PoolState, PoolStateV2>;

impl VerPoolState {
    #[inline]
    pub const fn try_from_acc_data(data: &[u8]) -> Option<Self> {
        if let Some(p) = PoolStatePacked::of_acc_data(data) {
            Some(Self::V1(p.into_pool_state()))
        } else {
            match PoolStateV2Packed::of_acc_data(data) {
                Some(p) => Some(Self::V2(p.into_pool_state_v2())),
                None => None,
            }
        }
    }

    #[inline]
    pub const fn total_sol_value(&self) -> u64 {
        *v1_2_each_field!(self, total_sol_value)
    }

    #[inline]
    pub const fn lp_token_mint(&self) -> &[u8; 32] {
        v1_2_each_field!(self, lp_token_mint)
    }

    #[inline]
    pub const fn pricing_program(&self) -> &[u8; 32] {
        v1_2_each_field!(self, pricing_program)
    }

    #[inline]
    pub const fn rebalance_authority(&self) -> &[u8; 32] {
        v1_2_each_field!(self, rebalance_authority)
    }

    #[inline]
    pub const fn is_rebalancing_mut(&mut self) -> &mut u8 {
        v1_2_each_field_mut!(self, is_rebalancing)
    }

    #[inline]
    pub const fn is_disabled_mut(&mut self) -> &mut u8 {
        v1_2_each_field_mut!(self, is_disabled)
    }

    #[inline]
    pub const fn as_acc_data_arr(&self) -> &[u8] {
        v1_2_each_meth!(self, as_acc_data_arr)
    }

    #[inline]
    pub const fn migrated(self, migration_slot: u64) -> PoolStateV2 {
        match self {
            Self::V2(p) => p,
            Self::V1(PoolState {
                total_sol_value,
                trading_protocol_fee_bps,
                lp_protocol_fee_bps,
                is_disabled,
                is_rebalancing,
                padding,
                admin,
                rebalance_authority,
                protocol_fee_beneficiary,
                pricing_program,
                lp_token_mint,
                version: _,
            }) => PoolStateV2 {
                total_sol_value,
                is_disabled,
                is_rebalancing,
                padding,
                admin,
                rebalance_authority,
                protocol_fee_beneficiary,
                pricing_program,
                lp_token_mint,
                protocol_fee_nanos: migrated_protocol_fee_nanos(
                    lp_protocol_fee_bps,
                    trading_protocol_fee_bps,
                ),
                rps: *Rps::DEFAULT.as_inner().as_raw(),
                rps_authority: admin,
                last_release_slot: migration_slot,
                version: 2,
                withheld_lamports: 0,
                protocol_fee_lamports: 0,
            },
        }
    }
}

// kinda dumb reimplementing the same logic in the program here again but
// serves as double-check i guess

const BPS_TO_NANOS_MULTIPLE: u32 = NANOS_DENOM / 10_000;

#[inline]
pub const fn migrated_protocol_fee_nanos(
    lp_protocol_fee_bps: u16,
    trading_protocol_fee_bps: u16,
) -> u32 {
    let max = if lp_protocol_fee_bps > trading_protocol_fee_bps {
        lp_protocol_fee_bps
    } else {
        trading_protocol_fee_bps
    };
    (max as u32) * BPS_TO_NANOS_MULTIPLE
}
