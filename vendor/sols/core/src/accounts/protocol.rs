use core::mem::{align_of, size_of};

use generic_array_struct::generic_array_struct;

use crate::{
    internal_utils::{impl_cast_from_acc_data, impl_cast_to_acc_data},
    keys::CONST_KEYS_OWNED,
    typedefs::{Nanos, ProtocolFlags, SvcWhitelist, SvcWhitelistMut},
    utils::ProtocolFeeRatio,
};

#[generic_array_struct(all pub)]
#[repr(transparent)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ProtocolV1Addrs<T> {
    pub fee_controller: T,
    pub beneficiary: T,
    pub svc_whitelist_auth: T,
}

pub type ProtocolV1AddrVals = ProtocolV1Addrs<[u8; 32]>;

pub const PROTOCOL_V1_ADDR_FNAMES: ProtocolV1Addrs<&'static str> =
    ProtocolV1Addrs::const_from_destr(ProtocolV1AddrsDestr {
        fee_controller: "fee_controller",
        beneficiary: "beneficiary",
        svc_whitelist_auth: "svc_whitelist_auth",
    });

pub const INIT_PROTOCOL_V1_ADDR_VALS: ProtocolV1AddrVals =
    ProtocolV1AddrVals::const_from_destr(ProtocolV1AddrsDestr {
        fee_controller: *CONST_KEYS_OWNED.init_protocol_fee_controller(),
        beneficiary: *CONST_KEYS_OWNED.init_protocol_beneficiary(),
        svc_whitelist_auth: *CONST_KEYS_OWNED.init_svc_whitelist_auth(),
    });

#[generic_array_struct(all pub)]
#[repr(transparent)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ProtocolV1Fees<T> {
    pub fixed: T,
    pub perf: T,
}

pub type ProtocolV1NanosRaw = ProtocolV1Fees<u32>;
pub type ProtocolV1NanosPked = ProtocolV1Fees<[u8; 4]>;
pub type ProtocolV1FeeRatios = ProtocolV1Fees<ProtocolFeeRatio>;

pub const PROTOCOL_V1_NANO_FNAMES: ProtocolV1Fees<&'static str> =
    ProtocolV1Fees::const_from_destr(ProtocolV1FeesDestr {
        fixed: "fixed",
        perf: "perf",
    });

pub const INIT_PROTOCOL_V1_NANOS: ProtocolV1NanosRaw =
    ProtocolV1NanosRaw::const_from_destr(ProtocolV1FeesDestr {
        // 10 bps
        fixed: 1_000_000,

        // 10%
        perf: 100_000_000,
    });

/// Protocol cannot take more than 10% of principal
pub const PROTOCOL_V1_MAX_FIXED_FEE_NANOS: Nanos = match Nanos::new(100_000_000) {
    Ok(x) => x,
    Err(_) => unreachable!(),
};

pub type ProtocolV1FeeLamports = ProtocolV1Fees<u64>;

impl ProtocolV1FeeLamports {
    /// Returns `None` on overflow
    #[inline]
    pub const fn total_checked(&self) -> Option<u64> {
        self.fixed().checked_add(*self.perf())
    }

    /// [`Self::total_checked`], but panics on overflow
    #[inline]
    pub const fn total(&self) -> u64 {
        self.total_checked().unwrap()
    }
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ProtocolV1<A, N, F, P> {
    /// [`ProtocolV1Addrs`]
    pub addrs: A,

    /// [`ProtocolV1Fees`]
    pub nanos: N,

    /// [`crate::typedefs::ProtocolFlags`]
    pub flags: F,

    pub padding: P,
}

pub const PROTOCOL_V1_PADDING_LEN: usize = 3;

/// Type to be used in onchain program.
///
/// Use primitive val (e.g. u32 instead of Nanos) to allow for lazy verification of
/// indiv fields as needed after unverified pointer casting.
pub type ProtocolV1Acc =
    ProtocolV1<ProtocolV1AddrVals, ProtocolV1NanosRaw, u8, [u8; PROTOCOL_V1_PADDING_LEN]>;
impl_cast_from_acc_data!(ProtocolV1Acc);
impl_cast_to_acc_data!(ProtocolV1Acc);

const _ASSERT_PROTOCOL_V1_ACC_NO_PADDING: () = assert!(
    size_of::<ProtocolV1Acc>()
        == size_of::<ProtocolV1AddrVals>()
            + size_of::<ProtocolV1NanosRaw>()
            + size_of::<u8>()
            + size_of::<[u8; PROTOCOL_V1_PADDING_LEN]>()
);
const _ASSERT_PROTOCOL_V1_ACC_AL: () = assert!(align_of::<ProtocolV1Acc>() == 4);

impl ProtocolV1Acc {
    pub const INIT: Self = Self {
        addrs: INIT_PROTOCOL_V1_ADDR_VALS,
        nanos: INIT_PROTOCOL_V1_NANOS,
        flags: *ProtocolFlags::V1.as_byte(),
        padding: [0; PROTOCOL_V1_PADDING_LEN],
    };

    /// # Safety
    /// - `acc_data` must have the same align as Self.
    #[inline]
    pub const unsafe fn of_acc_data_full(acc_data: &[u8]) -> Option<(&Self, &[[u8; 32]])> {
        let (this, wl) = match acc_data.split_first_chunk() {
            None => return None,
            Some(x) => x,
        };
        let this = unsafe { Self::of_acc_data_arr(this) };
        let wl = match SvcWhitelist::of_acc_data(wl) {
            None => return None,
            Some(x) => x,
        };
        Some((this, wl.0))
    }

    /// # Safety
    /// - `acc_data` must have the same align as Self.
    #[inline]
    pub const unsafe fn of_acc_data_full_mut(
        acc_data: &mut [u8],
    ) -> Option<(&mut Self, &mut [[u8; 32]])> {
        let (this, wl) = match acc_data.split_first_chunk_mut() {
            None => return None,
            Some(x) => x,
        };
        let this = unsafe { Self::of_acc_data_arr_mut(this) };
        let wl = match SvcWhitelistMut::of_acc_data(wl) {
            None => return None,
            Some(x) => x,
        };
        Some((this, wl.0))
    }
}

/// Packed version of [`ProtocolV1Acc`] (align == 1)
pub type ProtocolV1Pked =
    ProtocolV1<ProtocolV1AddrVals, ProtocolV1NanosPked, u8, [u8; PROTOCOL_V1_PADDING_LEN]>;
impl_cast_from_acc_data!(ProtocolV1Pked, packed);
impl_cast_to_acc_data!(ProtocolV1Pked, packed);
const _ASSERT_PROTOCOL_V1_PKED_SZ: () =
    assert!(size_of::<ProtocolV1Pked>() == size_of::<ProtocolV1Acc>());
const _ASSERT_PROTOCOL_V1_PKED_AL: () = assert!(align_of::<ProtocolV1Pked>() == 1);

impl ProtocolV1Pked {
    #[inline]
    pub const fn of_acc_data_full(acc_data: &[u8]) -> Option<(&Self, &[[u8; 32]])> {
        let (this, wl) = match acc_data.split_first_chunk() {
            None => return None,
            Some(x) => x,
        };
        let this = Self::of_acc_data_arr(this);
        let wl = match SvcWhitelist::of_acc_data(wl) {
            None => return None,
            Some(x) => x,
        };
        Some((this, wl.0))
    }
}

impl ProtocolV1Acc {
    #[inline]
    pub const fn into_pked(self) -> ProtocolV1Pked {
        let Self {
            addrs,
            nanos,
            flags,
            padding,
        } = self;
        let ProtocolV1FeesDestr { fixed, perf } = nanos.const_into_destr();
        ProtocolV1Pked {
            addrs,
            nanos: ProtocolV1Fees::const_from_destr(ProtocolV1FeesDestr {
                fixed: fixed.to_le_bytes(),
                perf: perf.to_le_bytes(),
            }),
            flags,
            padding,
        }
    }

    #[inline]
    pub const fn from_pked(
        ProtocolV1Pked {
            addrs,
            nanos,
            flags,
            padding,
        }: ProtocolV1Pked,
    ) -> Self {
        let ProtocolV1FeesDestr { fixed, perf } = nanos.const_into_destr();
        Self {
            addrs,
            nanos: ProtocolV1Fees::const_from_destr(ProtocolV1FeesDestr {
                fixed: u32::from_le_bytes(fixed),
                perf: u32::from_le_bytes(perf),
            }),
            flags,
            padding,
        }
    }
}

impl From<ProtocolV1Pked> for ProtocolV1Acc {
    #[inline]
    fn from(v: ProtocolV1Pked) -> Self {
        Self::from_pked(v)
    }
}

impl From<ProtocolV1Acc> for ProtocolV1Pked {
    #[inline]
    fn from(value: ProtocolV1Acc) -> Self {
        value.into_pked()
    }
}
