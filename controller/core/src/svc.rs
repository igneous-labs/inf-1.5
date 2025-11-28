use core::ops::RangeInclusive;

use inf1_svc_core::traits::{SolValCalc, SolValCalcAccs};
use sanctum_u64_ratio::{Floor, Ratio};

use crate::{err::Inf1CtlErr, typedefs::pool_sv::PoolSvLamports};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InfCalc {
    pub pool_lamports: PoolSvLamports,
    pub mint_supply: u64,
}

// "LST" in this context refers to INF

impl InfCalc {
    #[inline]
    pub const fn svc_lst_to_sol(&self, inf: u64) -> Result<RangeInclusive<u64>, Inf1CtlErr> {
        let n = match self.pool_lamports.lp_due_checked() {
            None => return Err(Inf1CtlErr::MathError),
            Some(n) => n,
        };
        match Floor(Ratio {
            n,
            d: self.mint_supply,
        })
        .apply(inf)
        {
            None => Err(Inf1CtlErr::MathError),
            Some(x) => Ok(x..=x),
        }
    }

    #[inline]
    pub const fn svc_sol_to_lst(&self, lamports: u64) -> Result<RangeInclusive<u64>, Inf1CtlErr> {
        let d = match self.pool_lamports.lp_due_checked() {
            None => return Err(Inf1CtlErr::MathError),
            Some(d) => d,
        };
        match Floor(Ratio {
            n: self.mint_supply,
            d,
        })
        .apply(lamports)
        {
            None => Err(Inf1CtlErr::MathError),
            Some(x) => Ok(x..=x),
        }
    }
}

impl SolValCalc for InfCalc {
    type Error = Inf1CtlErr;

    #[inline]
    fn lst_to_sol(&self, inf: u64) -> Result<RangeInclusive<u64>, Self::Error> {
        self.svc_lst_to_sol(inf)
    }

    #[inline]
    fn sol_to_lst(&self, lamports: u64) -> Result<RangeInclusive<u64>, Self::Error> {
        self.svc_sol_to_lst(lamports)
    }
}

/// The INF program does NOT implement the SOL value calculator program interface
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct InfDummyCalcAccs;

impl SolValCalcAccs for InfDummyCalcAccs {
    type KeysOwned = [[u8; 32]; 0];

    type AccFlags = [bool; 0];

    #[inline]
    fn suf_keys_owned(&self) -> Self::KeysOwned {
        []
    }

    #[inline]
    fn suf_is_writer(&self) -> Self::AccFlags {
        []
    }

    #[inline]
    fn suf_is_signer(&self) -> Self::AccFlags {
        []
    }
}
