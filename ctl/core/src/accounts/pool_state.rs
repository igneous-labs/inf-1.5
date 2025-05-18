use core::mem::size_of;

use crate::internal_utils::{impl_cast_from_acc_data, impl_cast_to_acc_data};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PoolState {
    pub total_sol_value: u64,
    pub trading_protocol_fee_bps: u16,
    pub lp_protocol_fee_bps: u16,
    pub version: u8,
    pub is_disabled: u8,
    pub is_rebalancing: u8,
    pub padding: [u8; 1],
    pub admin: [u8; 32],
    pub rebalance_authority: [u8; 32],
    pub protocol_fee_beneficiary: [u8; 32],
    pub pricing_program: [u8; 32],
    pub lp_token_mint: [u8; 32],
}
impl_cast_from_acc_data!(PoolState);
impl_cast_to_acc_data!(PoolState);

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PoolStatePacked {
    total_sol_value: [u8; 8],
    trading_protocol_fee_bps: [u8; 2],
    lp_protocol_fee_bps: [u8; 2],
    version: u8,
    is_disabled: u8,
    is_rebalancing: u8,
    padding: [u8; 1],
    admin: [u8; 32],
    rebalance_authority: [u8; 32],
    protocol_fee_beneficiary: [u8; 32],
    pricing_program: [u8; 32],
    lp_token_mint: [u8; 32],
}
impl_cast_from_acc_data!(PoolStatePacked, packed);
impl_cast_to_acc_data!(PoolStatePacked, packed);

impl PoolStatePacked {
    #[inline]
    pub const fn into_pool_state(self) -> PoolState {
        let Self {
            total_sol_value,
            trading_protocol_fee_bps,
            lp_protocol_fee_bps,
            version,
            is_disabled,
            is_rebalancing,
            padding,
            admin,
            rebalance_authority,
            protocol_fee_beneficiary,
            pricing_program,
            lp_token_mint,
        } = self;
        PoolState {
            total_sol_value: u64::from_le_bytes(total_sol_value),
            trading_protocol_fee_bps: u16::from_le_bytes(trading_protocol_fee_bps),
            lp_protocol_fee_bps: u16::from_le_bytes(lp_protocol_fee_bps),
            version,
            is_disabled,
            is_rebalancing,
            padding,
            admin,
            rebalance_authority,
            protocol_fee_beneficiary,
            pricing_program,
            lp_token_mint,
        }
    }
}

impl From<PoolStatePacked> for PoolState {
    #[inline]
    fn from(value: PoolStatePacked) -> Self {
        value.into_pool_state()
    }
}

const _ASSERT_PACKED_UNPACKED_SIZES_EQ: () =
    assert!(size_of::<PoolState>() == size_of::<PoolStatePacked>());
