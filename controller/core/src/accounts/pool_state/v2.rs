use core::mem::{align_of, size_of};

use generic_array_struct::generic_array_struct;

use crate::{
    accounts::pool_state::PoolState,
    err::{Inf1CtlErr, RpsOobErr},
    internal_utils::{
        impl_cast_from_acc_data, impl_cast_to_acc_data, impl_gas_memset, impl_verify_vers,
    },
    typedefs::{
        fee_nanos::{FeeNanos, FeeNanosTooLargeErr},
        rps::Rps,
        uq0f63::UQ0F63,
    },
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
impl_verify_vers!(PoolStateV2, 2);

impl PoolStateV2 {
    #[inline]
    pub const fn rps_checked(&self) -> Result<Rps, RpsOobErr> {
        let u = match UQ0F63::new(self.rps) {
            Err(e) => return Err(RpsOobErr::UQ0F63(e)),
            Ok(x) => x,
        };
        match Rps::new(u) {
            Err(e) => Err(RpsOobErr::Rps(e)),
            Ok(x) => Ok(x),
        }
    }

    #[inline]
    pub const fn protocol_fee_nanos_checked(&self) -> Result<FeeNanos, FeeNanosTooLargeErr> {
        FeeNanos::new(self.protocol_fee_nanos)
    }
}

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
impl_verify_vers!(PoolStateV2Packed, 2);

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
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PoolStateV2Addrs<T> {
    pub admin: T,
    pub rebalance_authority: T,
    pub protocol_fee_beneficiary: T,
    pub pricing_program: T,
    pub lp_token_mint: T,
    pub rps_authority: T,
}
impl_gas_memset!(PoolStateV2Addrs, POOL_STATE_V2_ADDRS_LEN);

#[generic_array_struct(builder pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PoolStateV2U64s<T> {
    pub total_sol_value: T,
    pub withheld_lamports: T,
    pub protocol_fee_lamports: T,
    pub last_release_slot: T,
    // rps excluded due to its different type
    // despite same repr
}
impl_gas_memset!(PoolStateV2U64s, POOL_STATE_V2U64S_LEN);

#[generic_array_struct(builder pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PoolStateV2U8Bools<T> {
    pub is_disabled: T,
    pub is_rebalancing: T,
}
impl_gas_memset!(PoolStateV2U8Bools, POOL_STATE_V2U8_BOOLS_LEN);

// TODO: if we were disciplined about packing all fields of the same type
// at the same region and didnt care about backward compatibility, then
// we could just use this type as the account data repr and woudlnt need
// conversion functions
/// Field-Type Aggregations
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PoolStateV2Fta<A, U, V, W, X> {
    pub addrs: PoolStateV2Addrs<A>,
    pub u64s: PoolStateV2U64s<U>,
    pub u8_bools: PoolStateV2U8Bools<V>,
    pub protocol_fee_nanos: W,
    pub rps: X,
}

pub type PoolStateV2FtaVals = PoolStateV2Fta<[u8; 32], u64, u8, FeeNanos, Rps>;

impl PoolStateV2FtaVals {
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

    #[inline]
    pub const fn try_from_pool_state_v2(ps: PoolStateV2) -> Result<Self, Inf1CtlErr> {
        let PoolStateV2 {
            total_sol_value,
            is_disabled,
            is_rebalancing,
            admin,
            rebalance_authority,
            protocol_fee_beneficiary,
            pricing_program,
            lp_token_mint,
            rps_authority,
            withheld_lamports,
            protocol_fee_lamports,
            last_release_slot,
            // explicitly list out unused fields to make sure we didnt miss any
            protocol_fee_nanos: _,
            version: _,
            padding: _,
            rps: _,
        } = ps;
        Ok(Self {
            addrs: PoolStateV2Addrs::memset([0; 32])
                .const_with_admin(admin)
                .const_with_lp_token_mint(lp_token_mint)
                .const_with_pricing_program(pricing_program)
                .const_with_protocol_fee_beneficiary(protocol_fee_beneficiary)
                .const_with_rebalance_authority(rebalance_authority)
                .const_with_rps_authority(rps_authority),
            u64s: PoolStateV2U64s::memset(0)
                .const_with_last_release_slot(last_release_slot)
                .const_with_protocol_fee_lamports(protocol_fee_lamports)
                .const_with_total_sol_value(total_sol_value)
                .const_with_withheld_lamports(withheld_lamports),
            u8_bools: PoolStateV2U8Bools::memset(0)
                .const_with_is_disabled(is_disabled)
                .const_with_is_rebalancing(is_rebalancing),
            protocol_fee_nanos: match ps.protocol_fee_nanos_checked() {
                Err(e) => return Err(Inf1CtlErr::FeeNanosOob(e)),
                Ok(x) => x,
            },
            rps: match ps.rps_checked() {
                Err(e) => return Err(Inf1CtlErr::RpsOob(e)),
                Ok(x) => x,
            },
        })
    }
}

impl From<PoolStateV2FtaVals> for PoolStateV2 {
    #[inline]
    fn from(value: PoolStateV2FtaVals) -> Self {
        value.into_pool_state_v2()
    }
}

impl TryFrom<PoolStateV2> for PoolStateV2FtaVals {
    type Error = Inf1CtlErr;

    #[inline]
    fn try_from(value: PoolStateV2) -> Result<Self, Self::Error> {
        Self::try_from_pool_state_v2(value)
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
