use core::{error::Error, fmt::Display, ops::RangeInclusive};

use generic_array_struct::generic_array_struct;
use inf1_svc_core::traits::SolValCalc;
use sanctum_sols_core::accounts::{PoolV1, PoolV1Acc, PoolV1LamportVals};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SolsCalc {
    pub pool_v1_lamport_vals: PoolV1LamportVals,
    pub u64s: SolsValcU64Vals,
}

#[generic_array_struct(all pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SolsCalcU64s<T> {
    pub pool_acc_lamports: T,
    pub mint_supply: T,
}

pub type SolsValcU64Vals = SolsCalcU64s<u64>;

/// Constructors
impl SolsCalc {
    #[inline]
    pub const fn new(PoolV1 { lamports, .. }: &PoolV1Acc, u64s: SolsValcU64Vals) -> Self {
        Self {
            pool_v1_lamport_vals: *lamports,
            u64s,
        }
    }
}

/// `SolValCalc`
///
/// Returns 1:1, erroring if the SOLS pool is insolvent for depositors.
///
/// NB: if the SOLS pool is currently out of SOL liquidity, the SOLS token might
/// only be redeemable for holding LSTs below par via ClaimHolding at that point in time.
impl SolsCalc {
    /// See [`sanctum_sols_core::utils::spec::is_pool_dep_solvent`]
    #[inline]
    pub const fn is_pool_dep_solvent(&self) -> bool {
        let dd = match self
            .pool_v1_lamport_vals
            .dep_due_checked(*self.u64s.pool_acc_lamports())
        {
            None => return false,
            Some(dd) => dd,
        };
        dd >= *self.u64s.mint_supply()
    }

    #[inline]
    pub const fn svc_lst_to_sol(
        &self,
        lst_amount: u64,
    ) -> Result<RangeInclusive<u64>, SolsCalcErr> {
        if self.is_pool_dep_solvent() {
            Ok(lst_amount..=lst_amount)
        } else {
            Err(SolsCalcErr::Insolvent)
        }
    }

    #[inline]
    pub const fn svc_sol_to_lst(
        &self,
        lamports_amount: u64,
    ) -> Result<RangeInclusive<u64>, SolsCalcErr> {
        if self.is_pool_dep_solvent() {
            Ok(lamports_amount..=lamports_amount)
        } else {
            Err(SolsCalcErr::Insolvent)
        }
    }
}

impl SolValCalc for SolsCalc {
    type Error = SolsCalcErr;

    #[inline]
    fn lst_to_sol(&self, lst_amount: u64) -> Result<RangeInclusive<u64>, Self::Error> {
        self.svc_lst_to_sol(lst_amount)
    }

    #[inline]
    fn sol_to_lst(&self, lamports_amount: u64) -> Result<RangeInclusive<u64>, Self::Error> {
        self.svc_sol_to_lst(lamports_amount)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SolsCalcErr {
    Insolvent,
}

impl SolsCalcErr {
    pub const INSOLVENT_ERR_STR: &str = "SOLS pool is insolvent";
}

impl Display for SolsCalcErr {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::Insolvent => Self::INSOLVENT_ERR_STR,
        })
    }
}

impl Error for SolsCalcErr {}
