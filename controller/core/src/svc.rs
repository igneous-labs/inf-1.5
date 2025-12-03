//! Calculation of SOL value / redemption rate of the LP token (INF)

use core::{error::Error, fmt::Display, ops::RangeInclusive};

use inf1_svc_core::traits::{SolValCalc, SolValCalcAccs};
use sanctum_u64_ratio::{Floor, Ratio};

use crate::{
    accounts::pool_state::PoolStateV2,
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
    #[inline]
    pub const fn new(pool_state_v2: &PoolStateV2, inf_mint_supply: u64) -> Self {
        Self {
            pool_lamports: PoolSvLamports::from_pool_state_v2(pool_state_v2),
            mint_supply: inf_mint_supply,
        }
    }

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

    /// Preserve same behaviour of edge-cases as v1:
    /// Calculate lp_token:sol_value at 1:1 exchange rate if
    ///
    /// - LP supply = 0.
    ///   If LP due sol value is not zero then this results in the first LP
    ///   geting all the orphaned sol value in the pool.  
    ///
    /// - LP due sol value = 0 & withheld = 0.
    ///   This results in the entering LP being immediately diluted by
    ///   existing INF token holders if the supply is nonzero.
    ///   Should never happen unless on catastrophic losses.
    ///   If withheld is nonzero, then zero will be returned;
    ///   value should be nonzero again after waiting some time.
    #[inline]
    pub const fn sol_to_inf(&self, sol: u64) -> Option<u64> {
        let r = match self.supply_over_lp_due() {
            None => return None,
            Some(r) => r,
        };
        let r = if r.n == 0 || (r.d == 0 && *self.pool_lamports.withheld() == 0) {
            Ratio::<u64, u64>::ONE
        } else {
            r
        };
        Floor(r).apply(sol)
    }

    /// Edge cases: same as [`Self::sol_to_inf`]
    #[inline]
    pub const fn inf_to_sol(&self, inf: u64) -> Option<u64> {
        let r = match self.lp_due_over_supply() {
            None => return None,
            Some(r) => r,
        };
        let r = if r.d == 0 || (r.n == 0 && *self.pool_lamports.withheld() == 0) {
            Ratio::<u64, u64>::ONE
        } else {
            r
        };
        Floor(r).apply(inf)
    }

    /// # Returns
    /// (inf_supply / lamports_due_to_lp)
    ///
    /// `None` if pool is insolvent for LPers (lp_due is negative)
    #[inline]
    pub const fn supply_over_lp_due(&self) -> Option<Ratio<u64, u64>> {
        match self.pool_lamports.lp_due_checked() {
            None => None,
            Some(d) => Some(Ratio {
                n: self.mint_supply,
                d,
            }),
        }
    }

    /// # Returns
    /// (lamports_due_to_lp / inf_supply)
    ///
    /// `None` if pool is insolvent for LPers (lp_due is negative)
    #[inline]
    pub const fn lp_due_over_supply(&self) -> Option<Ratio<u64, u64>> {
        match self.supply_over_lp_due() {
            None => None,
            // TODO: add .inv() to sanctum-u64-ratio
            Some(Ratio { n, d }) => Some(Ratio { n: d, d: n }),
        }
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
