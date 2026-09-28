use core::mem::{align_of, size_of};
use generic_array_struct::generic_array_struct;
use sanctum_u64_ratio::Ratio;

use crate::{
    internal_utils::{impl_cast_from_acc_data, impl_cast_to_acc_data},
    typedefs::Nanos,
};

#[generic_array_struct(all pub)]
#[repr(transparent)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PoolV1Addrs<T> {
    pub mint: T,
    pub admin: T,
    pub manager: T,
    pub rebalancer: T,
    pub rrr_controller: T,
}

pub type PoolV1AddrVals = PoolV1Addrs<[u8; 32]>;

pub const POOL_V1_ADDR_FNAMES: PoolV1Addrs<&'static str> =
    PoolV1Addrs::const_from_destr(PoolV1AddrsDestr {
        mint: "mint",
        admin: "admin",
        manager: "manager",
        rebalancer: "rebalancer",
        rrr_controller: "rrr_controller",
    });

#[generic_array_struct(all pub)]
#[repr(transparent)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PoolV1Lamports<T> {
    /// Invariant: always >0 to prevent deletion of account
    pub rent_exempt: T,
    pub outstanding: T,
    pub protocol_fee: T,
}

pub type PoolV1LamportVals = PoolV1Lamports<u64>;
pub type PoolV1LamportPked = PoolV1Lamports<[u8; 8]>;

pub const POOL_V1_LAMPORT_FNAMES: PoolV1Lamports<&'static str> =
    PoolV1Lamports::const_from_destr(PoolV1LamportsDestr {
        rent_exempt: "rent_exempt",
        outstanding: "outstanding",
        protocol_fee: "protocol_fee",
    });

impl PoolV1LamportVals {
    #[inline]
    pub const fn liq_lamport_vals(&self, pool_acc_lamports: u64) -> PoolLiqLamportVals {
        PoolLiqLamportVals::const_from_destr(PoolLiqLamportsDestr {
            total: pool_acc_lamports,
            rent_exempt: *self.rent_exempt(),
        })
    }

    /// SOL liquidity available for Claiming
    #[inline]
    pub const fn liq_avail_checked(&self, pool_acc_lamports: u64) -> Option<u64> {
        self.liq_lamport_vals(pool_acc_lamports).liq_avail_checked()
    }

    /// Useful offchain or if invariants mentioned in [`Self::liq_avail_checked`]
    /// are known beforehand to be met
    #[inline]
    pub const fn liq_avail(&self, pool_acc_lamports: u64) -> u64 {
        self.liq_lamport_vals(pool_acc_lamports).liq_avail()
    }

    /// Total lamports due to depositors
    ///
    /// # Returns
    /// - `None` on overflow
    #[inline]
    pub const fn dep_due_checked(&self, pool_acc_lamports: u64) -> Option<u64> {
        // order of ops matter here: picked specifically to minimize odds of overflow

        let x = match self.liq_avail_checked(pool_acc_lamports) {
            None => return None,
            Some(x) => x,
        };

        // total lamports in circulation on all of solana should not
        // exceed u64::MAX so this addition should not overflow
        let x = match x.checked_add(*self.outstanding()) {
            None => return None,
            Some(x) => x,
        };

        // pool always solvent for depositor invariant means this subtraction
        // should not overflow
        x.checked_sub(*self.protocol_fee())
    }

    /// Useful offchain or if invariants mentioned in [`Self::dep_due_checked`]
    /// are known beforehand to be met
    #[inline]
    pub const fn dep_due(&self, pool_acc_lamports: u64) -> u64 {
        match self.dep_due_checked(pool_acc_lamports) {
            None => panic!("dep_due_checked None"),
            Some(x) => x,
        }
    }

    /// Returns the pool's current reserve ratio
    #[inline]
    pub const fn rr_checked(&self, pool_acc_lamports: u64) -> Option<Ratio<u64, u64>> {
        let n = match self.liq_avail_checked(pool_acc_lamports) {
            None => return None,
            Some(x) => x,
        };
        let d = match self.dep_due_checked(pool_acc_lamports) {
            None => return None,
            Some(x) => x,
        };
        Some(Ratio { n, d })
    }

    /// Useful offchain or if invariants mentioned in [`Self::rr_checked`]
    /// are known beforehand to be met
    #[inline]
    pub const fn rr(&self, pool_acc_lamports: u64) -> Ratio<u64, u64> {
        match self.rr_checked(pool_acc_lamports) {
            None => panic!("rr_checked None"),
            Some(x) => x,
        }
    }
}

/// The pool account's lamport values. Used mainly to calculate liquid
/// SOL available for claiming
#[generic_array_struct(all pub)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PoolLiqLamports<T> {
    /// account.lamports
    pub total: T,

    /// `pool.lamports.rent_exempt()`
    pub rent_exempt: T,
}

pub type PoolLiqLamportVals = PoolLiqLamports<u64>;

impl PoolLiqLamportVals {
    /// SOL liquidity available for Claiming
    ///
    /// # Returns
    /// `None` if total < self.rent_exempt()
    #[inline]
    pub const fn liq_avail_checked(&self) -> Option<u64> {
        // invariant total acc lamports >= rent_exemption should be held
        // so this subtraction should not overflow
        self.total().checked_sub(*self.rent_exempt())
    }

    /// Useful offchain or if invariants mentioned in [`Self::liq_avail_checked`]
    /// are known beforehand to be met
    #[inline]
    pub const fn liq_avail(&self) -> u64 {
        match self.liq_avail_checked() {
            None => panic!("liq_avail_checked None"),
            Some(x) => x,
        }
    }
}

#[generic_array_struct(all pub)]
#[repr(transparent)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PoolV1Nanos<T> {
    pub rrr_floor: T,
    pub rrr_ceil: T,
    pub sol_out_loss_tol: T,
}

pub const INIT_RRR_FLOOR: u32 = 0;
pub const INIT_RRR_CEIL: u32 = 0;
pub const INIT_SOL_OUT_LOSS_TOL: u32 = 0;

pub type PoolV1NanosRaw = PoolV1Nanos<u32>;

impl PoolV1NanosRaw {
    /// All zeros
    pub const INIT: Self = Self::const_from_destr(PoolV1NanosDestr {
        rrr_floor: INIT_RRR_FLOOR,
        rrr_ceil: INIT_RRR_CEIL,
        sol_out_loss_tol: INIT_SOL_OUT_LOSS_TOL,
    });
}

pub type PoolV1NanosPked = PoolV1Nanos<[u8; 4]>;

pub const POOL_V1_NANO_FNAMES: PoolV1Nanos<&'static str> =
    PoolV1Nanos::const_from_destr(PoolV1NanosDestr {
        rrr_floor: "rrr_floor",
        rrr_ceil: "rrr_ceil",
        sol_out_loss_tol: "sol_out_loss_tol",
    });

/// 1%
pub const POOL_V1_MAX_SOL_OUT_LOSS_TOL: Nanos = match Nanos::new(10_000_000) {
    Ok(x) => x,
    Err(_) => unreachable!(),
};

#[generic_array_struct(all pub)]
#[repr(transparent)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PoolV1Pdas<T> {
    pub wsol_bridge: T,
    pub portfolio: T,
    pub pool: T,
}

pub type PoolV1PdaBumps = PoolV1Pdas<u8>;

pub const POOL_V1_PDA_FNAMES: PoolV1Pdas<&'static str> =
    PoolV1Pdas::const_from_destr(PoolV1PdasDestr {
        wsol_bridge: "wsol_bridge",
        portfolio: "portfolio",
        pool: "pool",
    });

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PoolV1<A, L, N, B, F> {
    /// [`PoolV1Addrs`]
    pub addrs: A,

    /// [`PoolV1Lamports`]
    pub lamports: L,

    /// [`PoolV1Nanos`]
    pub nanos: N,

    /// [`PoolV1Pdas`]
    pub bumps: B,

    /// [`crate::typedefs::PoolFlags`]
    pub flags: F,
}

/// Type to be used in onchain program.
///
/// Use primitive val (e.g. u32 instead of Nanos) to allow for lazy verification of
/// indiv fields as needed after unverified pointer casting.
pub type PoolV1Acc = PoolV1<PoolV1AddrVals, PoolV1LamportVals, PoolV1NanosRaw, PoolV1PdaBumps, u8>;
impl_cast_from_acc_data!(PoolV1Acc);
impl_cast_to_acc_data!(PoolV1Acc);

const _ASSERT_POOL_V1_ACC_NO_PADDING: () = assert!(
    size_of::<PoolV1Acc>()
        == size_of::<PoolV1AddrVals>()
            + size_of::<PoolV1LamportVals>()
            + size_of::<PoolV1NanosRaw>()
            + size_of::<PoolV1PdaBumps>()
            + size_of::<u8>()
);
const _ASSERT_POOL_V1_ACC_AL: () = assert!(align_of::<PoolV1Acc>() == 8);

/// Condition must hold for TransferWrappedThenMint's direct lamport transfer to init
/// wsol_bridge to occur without issue
const _ASSERT_POOL_V1_ACC_SZ_GTE_TOKEN_ACC: () = assert!(size_of::<PoolV1Acc>() >= 165);

impl PoolV1Acc {
    /// [`PoolV1LamportVals::dep_due_checked`]
    #[inline]
    pub const fn dep_due_checked(&self, pool_acc_lamports: u64) -> Option<u64> {
        self.lamports.dep_due_checked(pool_acc_lamports)
    }

    /// [`PoolV1LamportVals::dep_due`]
    #[inline]
    pub const fn dep_due(&self, pool_acc_lamports: u64) -> u64 {
        self.lamports.dep_due(pool_acc_lamports)
    }
}

/// Packed version of [`PoolV1`] (align == 1)
pub type PoolV1Pked =
    PoolV1<PoolV1AddrVals, PoolV1LamportPked, PoolV1NanosPked, PoolV1PdaBumps, u8>;
impl_cast_from_acc_data!(PoolV1Pked, packed);
impl_cast_to_acc_data!(PoolV1Pked, packed);
const _ASSERT_POOL_V1_PKED_SZ: () = assert!(size_of::<PoolV1Pked>() == size_of::<PoolV1Acc>());
const _ASSERT_POOL_V1_PKED_AL: () = assert!(align_of::<PoolV1Pked>() == 1);

impl PoolV1Acc {
    #[inline]
    pub const fn into_pked(self) -> PoolV1Pked {
        // TODO: gas: if only theres a way to do a `fn const_map()`
        let Self {
            addrs,
            lamports,
            nanos,
            bumps,
            flags,
        } = self;
        let PoolV1LamportsDestr {
            rent_exempt,
            outstanding,
            protocol_fee,
        } = lamports.const_into_destr();
        let PoolV1NanosDestr {
            rrr_floor,
            rrr_ceil,
            sol_out_loss_tol,
        } = nanos.const_into_destr();
        PoolV1Pked {
            addrs,
            lamports: PoolV1Lamports::const_from_destr(PoolV1LamportsDestr {
                rent_exempt: rent_exempt.to_le_bytes(),
                outstanding: outstanding.to_le_bytes(),
                protocol_fee: protocol_fee.to_le_bytes(),
            }),
            nanos: PoolV1Nanos::const_from_destr(PoolV1NanosDestr {
                rrr_floor: rrr_floor.to_le_bytes(),
                rrr_ceil: rrr_ceil.to_le_bytes(),
                sol_out_loss_tol: sol_out_loss_tol.to_le_bytes(),
            }),
            bumps,
            flags,
        }
    }

    #[inline]
    pub const fn from_pked(
        PoolV1Pked {
            addrs,
            lamports,
            nanos,
            bumps,
            flags,
        }: PoolV1Pked,
    ) -> Self {
        let PoolV1LamportsDestr {
            rent_exempt,
            outstanding,
            protocol_fee,
        } = lamports.const_into_destr();
        let PoolV1NanosDestr {
            rrr_floor,
            rrr_ceil,
            sol_out_loss_tol,
        } = nanos.const_into_destr();
        Self {
            addrs,
            lamports: PoolV1Lamports::const_from_destr(PoolV1LamportsDestr {
                rent_exempt: u64::from_le_bytes(rent_exempt),
                outstanding: u64::from_le_bytes(outstanding),
                protocol_fee: u64::from_le_bytes(protocol_fee),
            }),
            nanos: PoolV1Nanos::const_from_destr(PoolV1NanosDestr {
                rrr_floor: u32::from_le_bytes(rrr_floor),
                rrr_ceil: u32::from_le_bytes(rrr_ceil),
                sol_out_loss_tol: u32::from_le_bytes(sol_out_loss_tol),
            }),
            bumps,
            flags,
        }
    }
}

impl From<PoolV1Pked> for PoolV1Acc {
    #[inline]
    fn from(v: PoolV1Pked) -> Self {
        Self::from_pked(v)
    }
}

impl From<PoolV1Acc> for PoolV1Pked {
    #[inline]
    fn from(v: PoolV1Acc) -> Self {
        v.into_pked()
    }
}
