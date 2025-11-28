//! Calculation of SOL value / redemption rate of the LP token (INF)

use core::{error::Error, fmt::Display, ops::RangeInclusive};

use inf1_svc_core::traits::{SolValCalc, SolValCalcAccs};
use sanctum_u64_ratio::{Floor, Ratio};

use crate::{
    err::Inf1CtlErr,
    typedefs::pool_sv::{PoolSv, PoolSvLamports},
    yields::release::{ReleaseYield, ReleaseYieldParams},
};

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

impl InfCalc {
    /// Returns what `self` would be in the future given the yield release event
    ///
    /// # Returns
    /// `None` if `apply_yrel` returns None
    #[inline]
    pub fn lookahead(mut self, params: ReleaseYieldParams) -> Option<Self> {
        let yrel = ReleaseYield {
            params,
            withheld_lamports: *self.pool_lamports.withheld(),
        }
        .calc();
        PoolSv(self.pool_lamports.0.each_mut()).apply_yrel(yrel)?;
        Some(self)
    }

    #[inline]
    pub const fn inf_to_sol(&self, inf: u64) -> Option<u64> {
        let n = match self.pool_lamports.lp_due_checked() {
            None => return None,
            Some(n) => n,
        };
        Floor(Ratio {
            n,
            d: self.mint_supply,
        })
        .apply(inf)
    }

    #[inline]
    pub const fn sol_to_inf(&self, sol: u64) -> Option<u64> {
        let d = match self.pool_lamports.lp_due_checked() {
            None => return None,
            Some(d) => d,
        };
        Floor(Ratio {
            n: self.mint_supply,
            d,
        })
        .apply(sol)
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

/// SolValCalc traits const adapters
impl InfCalc {
    #[inline]
    pub const fn svc_lst_to_sol(&self, inf: u64) -> Result<RangeInclusive<u64>, InfCalcErr> {
        match self.inf_to_sol(inf) {
            None => Err(InfCalcErr::Math),
            Some(x) => Ok(x..=x),
        }
    }

    #[inline]
    pub const fn svc_sol_to_lst(&self, sol: u64) -> Result<RangeInclusive<u64>, InfCalcErr> {
        match self.sol_to_inf(sol) {
            None => Err(InfCalcErr::Math),
            Some(x) => Ok(x..=x),
        }
    }
}

/// # Notes
/// - the values returned by this SolValCalc do not take into account withdrawal (removeLiquidity) fees,
///   unlike the other stake pools' ones
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
