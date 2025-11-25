use core::mem::{align_of, size_of};

use generic_array_struct::generic_array_struct;

use crate::{
    accounts::pool_state::PoolState,
    internal_utils::{impl_cast_from_acc_data, impl_cast_to_acc_data},
    typedefs::{fee_nanos::FeeNanos, rps::Rps},
};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PoolStateV2 {
    pub total_sol_value: u64,

    // combined from `trading_protocol_fee_bps`
    // and `lp_protocol_fee_bps` in v1
    pub protocol_fee_nanos: u32,

    pub version: u8,
    pub is_disabled: u8,
    pub is_rebalancing: u8,
    pub padding: [u8; 1],
    pub admin: [u8; 32],
    pub rebalance_authority: [u8; 32],
    pub protocol_fee_beneficiary: [u8; 32],
    pub pricing_program: [u8; 32],
    pub lp_token_mint: [u8; 32],

    // new fields added over V1
    pub rps_authority: [u8; 32],
    pub rps: u64,
    pub withheld_lamports: u64,
    pub protocol_fee_lamports: u64,
    pub last_release_slot: u64,
}
impl_cast_from_acc_data!(PoolStateV2);
impl_cast_to_acc_data!(PoolStateV2);

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PoolStateV2Packed {
    total_sol_value: [u8; 8],
    protocol_fee_nanos: [u8; 4],
    version: u8,
    is_disabled: u8,
    is_rebalancing: u8,
    padding: [u8; 1],
    admin: [u8; 32],
    rebalance_authority: [u8; 32],
    protocol_fee_beneficiary: [u8; 32],
    pricing_program: [u8; 32],
    lp_token_mint: [u8; 32],
    rps_authority: [u8; 32],
    rps: [u8; 8],
    withheld_lamports: [u8; 8],
    protocol_fee_lamports: [u8; 8],
    last_release_slot: [u8; 8],
}
impl_cast_from_acc_data!(PoolStateV2Packed, packed);
impl_cast_to_acc_data!(PoolStateV2Packed, packed);

impl PoolStateV2Packed {
    #[inline]
    pub const fn into_pool_state_v2(self) -> PoolStateV2 {
        let Self {
            total_sol_value,
            protocol_fee_nanos,
            version,
            is_disabled,
            is_rebalancing,
            padding,
            admin,
            rebalance_authority,
            protocol_fee_beneficiary,
            pricing_program,
            lp_token_mint,
            withheld_lamports,
            protocol_fee_lamports,
            last_release_slot,
            rps,
            rps_authority,
        } = self;
        PoolStateV2 {
            total_sol_value: u64::from_le_bytes(total_sol_value),
            protocol_fee_nanos: u32::from_le_bytes(protocol_fee_nanos),
            version,
            is_disabled,
            is_rebalancing,
            padding,
            admin,
            rebalance_authority,
            protocol_fee_beneficiary,
            pricing_program,
            lp_token_mint,
            withheld_lamports: u64::from_le_bytes(withheld_lamports),
            protocol_fee_lamports: u64::from_le_bytes(protocol_fee_lamports),
            last_release_slot: u64::from_le_bytes(last_release_slot),
            rps: u64::from_le_bytes(rps),
            rps_authority,
        }
    }

    /// # Safety
    /// - `self` must be pointing to mem that has same align as `PoolState`.
    ///   This is true onchain for a PoolState account since account data
    ///   is always 8-byte aligned onchain.
    #[inline]
    pub const unsafe fn as_pool_state_v2(&self) -> &PoolStateV2 {
        &*(self as *const Self).cast()
    }

    /// # Safety
    /// - same rules as [`Self::as_pool_state`] apply
    #[inline]
    pub const unsafe fn as_pool_state_v2_mut(&mut self) -> &mut PoolStateV2 {
        &mut *(self as *mut Self).cast()
    }
}

impl From<PoolStateV2Packed> for PoolStateV2 {
    #[inline]
    fn from(value: PoolStateV2Packed) -> Self {
        value.into_pool_state_v2()
    }
}

// field type aggregations
// NB: v1's are in test-utils but for v2, we move it into core since they may be
// generally useful, and also so that they can be used for unit tests

#[generic_array_struct(builder pub)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PoolStateV2Addrs<T> {
    pub admin: T,
    pub rebalance_authority: T,
    pub protocol_fee_beneficiary: T,
    pub pricing_program: T,
    pub lp_token_mint: T,
    pub rps_authority: T,
}

#[generic_array_struct(builder pub)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PoolStateV2U64s<T> {
    pub total_sol_value: T,
    pub withheld_lamports: T,
    pub protocol_fee_lamports: T,
    pub last_release_slot: T,
    // rps excluded due to its different type
    // despite same repr
}

#[generic_array_struct(builder pub)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PoolStateV2U8Bools<T> {
    pub is_disabled: T,
    pub is_rebalancing: T,
}

// TODO: if we were disciplined about packing all fields of the same type
// at the same region and didnt care about backward compatibility, then
// we could just use this type as the account data repr and woudlnt need
// conversion functions
/// Field-Type aggregations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PoolStateV2FT<A, U, V, W, X> {
    pub addrs: PoolStateV2Addrs<A>,
    pub u64s: PoolStateV2U64s<U>,
    pub u8_bools: PoolStateV2U8Bools<V>,
    pub protocol_fee_nanos: W,
    pub rps: X,
}

pub type PoolStateV2FTVals = PoolStateV2FT<[u8; 32], u64, u8, FeeNanos, Rps>;

impl PoolStateV2FTVals {
    #[inline]
    pub const fn into_pool_state_v2(self) -> PoolStateV2 {
        let Self {
            addrs,
            u64s,
            u8_bools,
            protocol_fee_nanos,
            rps,
        } = self;
        PoolStateV2 {
            total_sol_value: *u64s.total_sol_value(),
            protocol_fee_nanos: protocol_fee_nanos.get(),
            version: 2u8,
            is_disabled: *u8_bools.is_disabled(),
            is_rebalancing: *u8_bools.is_rebalancing(),
            padding: [0u8],
            admin: *addrs.admin(),
            rebalance_authority: *addrs.rebalance_authority(),
            protocol_fee_beneficiary: *addrs.protocol_fee_beneficiary(),
            pricing_program: *addrs.pricing_program(),
            lp_token_mint: *addrs.lp_token_mint(),
            rps_authority: *addrs.rps_authority(),
            rps: *rps.as_inner().as_raw(),
            withheld_lamports: *u64s.withheld_lamports(),
            protocol_fee_lamports: *u64s.protocol_fee_lamports(),
            last_release_slot: *u64s.last_release_slot(),
        }
    }
}

impl From<PoolStateV2FTVals> for PoolStateV2 {
    #[inline]
    fn from(value: PoolStateV2FTVals) -> Self {
        value.into_pool_state_v2()
    }
}

const _ASSERT_PACKED_UNPACKED_SIZES_EQ: () =
    assert!(size_of::<PoolStateV2>() == size_of::<PoolStateV2Packed>());

const _ASSERT_SAME_ALIGN_AS_V1: () = assert!(align_of::<PoolStateV2>() == align_of::<PoolState>());

/// Check we didn't mess up existing fields from v1
/// `assert_offset_unchanged`
macro_rules! aou {
    ($ASSERTION:ident, $field:ident) => {
        const $ASSERTION: () = assert!(
            core::mem::offset_of!(PoolStateV2, $field) == core::mem::offset_of!(PoolState, $field)
        );
    };
}

aou!(_TOTAL_SOL_VALUE, total_sol_value);
aou!(_VERSION, version);
aou!(_IS_DISABLED, is_disabled);
aou!(_IS_REBALANCING, is_rebalancing);
aou!(_PADDING, padding);
aou!(_ADMIN, admin);
aou!(_REBALANCE_AUTH, rebalance_authority);
aou!(_PROTOCOL_FEE_BENEFICIARY, protocol_fee_beneficiary);
aou!(_PRICING_PROGRAM, pricing_program);
aou!(_LP_TOKEN_MINT, lp_token_mint);
