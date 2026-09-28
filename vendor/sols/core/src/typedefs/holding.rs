use core::mem::{align_of, size_of};

use generic_array_struct::generic_array_struct;

use crate::internal_utils::{impl_cast_from_acc_data, impl_cast_to_acc_data};

#[generic_array_struct(all pub)]
#[repr(transparent)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HoldingV1Addrs<T> {
    pub mint: T,

    /// SOL value calculator program
    pub svc: T,
}

pub type HoldingV1AddrVals = HoldingV1Addrs<[u8; 32]>;

pub const HOLDING_V1_ADDR_FNAMES: HoldingV1Addrs<&'static str> =
    HoldingV1Addrs::const_from_destr(HoldingV1AddrsDestr {
        mint: "mint",
        svc: "svc",
    });

#[generic_array_struct(all pub)]
#[repr(transparent)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HoldingV1Lamports<T> {
    pub outstanding: T,
    pub sol_value: T,
}

pub type HoldingV1LamportVals = HoldingV1Lamports<u64>;
pub type HoldingV1LamportPked = HoldingV1Lamports<[u8; 8]>;

pub const HOLDING_V1_LAMPORT_FNAMES: HoldingV1Lamports<&'static str> =
    HoldingV1Lamports::const_from_destr(HoldingV1LamportsDestr {
        outstanding: "outstanding",
        sol_value: "sol_value",
    });

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HoldingV1<A, L, N, F, B, P> {
    /// [`HoldingV1Addrs`]
    pub addrs: A,

    /// [`HoldingV1Lamports`]
    pub lamports: L,

    /// [`crate::typedefs::Nanos`]
    pub deposit_fee_nanos: N,

    /// [`crate::typedefs::HoldingFlags`]
    pub flags: F,
    pub ata_bump: B,
    pub padding: P,
}

pub const HOLDING_V1_PADDING_LEN: usize = 2;

/// Type to be used in packed list accounts.
///
/// Use primitive values for flags/bump for cheap verification after casting.
pub type HoldingV1Entry =
    HoldingV1<HoldingV1AddrVals, HoldingV1LamportVals, u32, u8, u8, [u8; HOLDING_V1_PADDING_LEN]>;
impl_cast_from_acc_data!(HoldingV1Entry);
impl_cast_to_acc_data!(HoldingV1Entry);
const _ASSERT_HOLDING_V1_ACC_NO_PADDING: () = assert!(
    size_of::<HoldingV1Entry>()
        == size_of::<HoldingV1AddrVals>()
            + size_of::<HoldingV1LamportVals>()
            + size_of::<u32>()
            + size_of::<u8>()
            + size_of::<u8>()
            + size_of::<[u8; HOLDING_V1_PADDING_LEN]>()
);
const _ASSERT_HOLDING_V1_ACC_AL: () = assert!(align_of::<HoldingV1Entry>() == 8);

/// Packed version of [`HoldingV1Acc`] (align == 1)
pub type HoldingV1Pked = HoldingV1<
    HoldingV1AddrVals,
    HoldingV1LamportPked,
    [u8; 4],
    u8,
    u8,
    [u8; HOLDING_V1_PADDING_LEN],
>;
impl_cast_from_acc_data!(HoldingV1Pked, packed);
impl_cast_to_acc_data!(HoldingV1Pked, packed);
const _ASSERT_HOLDING_V1_PKED_SZ: () =
    assert!(size_of::<HoldingV1Pked>() == size_of::<HoldingV1Entry>());
const _ASSERT_HOLDING_V1_PKED_AL: () = assert!(align_of::<HoldingV1Pked>() == 1);

impl HoldingV1Entry {
    #[inline]
    pub const fn into_pked(self) -> HoldingV1Pked {
        let Self {
            addrs,
            lamports,
            deposit_fee_nanos,
            flags,
            ata_bump,
            padding,
        } = self;
        let HoldingV1LamportsDestr {
            outstanding,
            sol_value,
        } = lamports.const_into_destr();
        HoldingV1Pked {
            addrs,
            lamports: HoldingV1Lamports::const_from_destr(HoldingV1LamportsDestr {
                outstanding: outstanding.to_le_bytes(),
                sol_value: sol_value.to_le_bytes(),
            }),
            deposit_fee_nanos: deposit_fee_nanos.to_le_bytes(),
            flags,
            ata_bump,
            padding,
        }
    }

    #[inline]
    pub const fn from_pked(
        HoldingV1Pked {
            addrs,
            lamports,
            deposit_fee_nanos,
            flags,
            ata_bump,
            padding,
        }: HoldingV1Pked,
    ) -> Self {
        let HoldingV1LamportsDestr {
            outstanding,
            sol_value,
        } = lamports.const_into_destr();
        Self {
            addrs,
            lamports: HoldingV1Lamports::const_from_destr(HoldingV1LamportsDestr {
                outstanding: u64::from_le_bytes(outstanding),
                sol_value: u64::from_le_bytes(sol_value),
            }),
            deposit_fee_nanos: u32::from_le_bytes(deposit_fee_nanos),
            flags,
            ata_bump,
            padding,
        }
    }
}

impl From<HoldingV1Pked> for HoldingV1Entry {
    #[inline]
    fn from(v: HoldingV1Pked) -> Self {
        Self::from_pked(v)
    }
}

impl From<HoldingV1Entry> for HoldingV1Pked {
    #[inline]
    fn from(value: HoldingV1Entry) -> Self {
        value.into_pked()
    }
}
