#![cfg_attr(not(test), no_std)]

use core::{error::Error, fmt::Display, ops::RangeInclusive};

use inf1_svc_core::traits::{SolValCalc, SolValCalcAccs};
use sanctum_u64_ratio::{Floor, Ratio};

use inf1_ctl_core::{
    err::Inf1CtlErr,
    typedefs::{fee_nanos::FeeNanos, pool_sv::PoolSvLamports, rps::Rps},
};

// Re-exports
pub use inf1_ctl_core::keys::POOL_STATE_ID;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InfCalc {
    pub pool_lamports: PoolSvLamports,
    pub mint_supply: u64,
}

impl InfCalc {
    pub const DEFAULT: Self = Self {
        pool_lamports: PoolSvLamports::memset(0),
        mint_supply: 0,
    };
}

impl Default for InfCalc {
    #[inline]
    fn default() -> Self {
        Self::DEFAULT
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InfCalcLookAhead {
    pub curr_slot: u64,
    pub pool_rps: Rps,
    pub pool_last_release_slot: u64,
    pub pool_protocol_fee_nanos: FeeNanos,
}

// "LST" in this context refers to INF

impl InfCalc {
    #[inline]
    pub const fn svc_lst_to_sol(&self, inf: u64) -> Result<RangeInclusive<u64>, InfCalcErr> {
        let n = match self.pool_lamports.lp_due_checked() {
            None => return Err(InfCalcErr::Math),
            Some(n) => n,
        };
        match Floor(Ratio {
            n,
            d: self.mint_supply,
        })
        .apply(inf)
        {
            None => Err(InfCalcErr::Math),
            Some(x) => Ok(x..=x),
        }
    }

    #[inline]
    pub const fn svc_sol_to_lst(&self, lamports: u64) -> Result<RangeInclusive<u64>, InfCalcErr> {
        let d = match self.pool_lamports.lp_due_checked() {
            None => return Err(InfCalcErr::Math),
            Some(d) => d,
        };
        match Floor(Ratio {
            n: self.mint_supply,
            d,
        })
        .apply(lamports)
        {
            None => Err(InfCalcErr::Math),
            Some(x) => Ok(x..=x),
        }
    }
}

impl SolValCalc for InfCalc {
    type Error = InfCalcErr;

    #[inline]
    fn lst_to_sol(&self, inf: u64) -> Result<RangeInclusive<u64>, Self::Error> {
        self.svc_lst_to_sol(inf)
    }

    #[inline]
    fn sol_to_lst(&self, lamports: u64) -> Result<RangeInclusive<u64>, Self::Error> {
        self.svc_sol_to_lst(lamports)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub enum InfCalcErr {
    #[default]
    Math,
}

impl Display for InfCalcErr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Math => f.write_str("MathError"),
        }
    }
}

impl Error for InfCalcErr {}

impl From<InfCalcErr> for Inf1CtlErr {
    fn from(e: InfCalcErr) -> Self {
        match e {
            InfCalcErr::Math => Self::MathError,
        }
    }
}

/// The INF program does NOT implement the SOL value calculator program interface.
///
/// We also do not need to pass in any additional calc accounts for getting its sol value
/// since all swap instructions that involve minting/burning INF would already include
/// the required 2 accounts, PoolState and INF mint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct InfDummyCalcAccs;

impl InfDummyCalcAccs {
    #[inline]
    pub const fn svc_suf_keys_owned(&self) -> <Self as SolValCalcAccs>::KeysOwned {
        []
    }

    #[inline]
    pub const fn svc_suf_is_writer(&self) -> <Self as SolValCalcAccs>::AccFlags {
        []
    }

    #[inline]
    pub const fn svc_suf_is_signer(&self) -> <Self as SolValCalcAccs>::AccFlags {
        []
    }
}

impl SolValCalcAccs for InfDummyCalcAccs {
    type KeysOwned = [[u8; 32]; 0];

    type AccFlags = [bool; 0];

    #[inline]
    fn suf_keys_owned(&self) -> Self::KeysOwned {
        self.svc_suf_keys_owned()
    }

    #[inline]
    fn suf_is_writer(&self) -> Self::AccFlags {
        self.svc_suf_is_writer()
    }

    #[inline]
    fn suf_is_signer(&self) -> Self::AccFlags {
        self.svc_suf_is_signer()
    }
}
