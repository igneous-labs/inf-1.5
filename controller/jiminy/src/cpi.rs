use core::ops::RangeInclusive;

use inf1_ctl_core::instructions::sync_sol_value::SyncSolValueIxPreAccs;
use inf1_pp_core::{
    instructions::price::{exact_in::PriceExactInIxArgs, exact_out::PriceExactOutIxArgs},
    traits::main::{PriceExactIn, PriceExactOut},
};
use inf1_svc_core::traits::SolValCalc;
use jiminy_cpi::account::AccountHandle;
use jiminy_program_error::{ProgramError, CUSTOM_ZERO};

/// `S: AsRef<[AccountHandle]>`
/// -> use [`IxAccountHandles::seq`] with [`jiminy_cpi::Cpi::invoke_fwd`]
pub type SyncSolValueIxPreAccountHandles<'account> = SyncSolValueIxPreAccs<AccountHandle<'account>>;

// TODO: make invoke() helpers for client programs

/// Wrapper for the return value from CPI call to `sol-val-calc` program
///
/// This is then used to implement the `SolValCalc` trait
/// so as to have re-use the same `quote_*` functions
///
pub struct LstToSolRetVal(pub RangeInclusive<u64>);
pub struct SolToLstRetVal(pub RangeInclusive<u64>);

impl SolValCalc for LstToSolRetVal {
    type Error = ProgramError;

    fn lst_to_sol(&self, _lst_amount: u64) -> Result<RangeInclusive<u64>, Self::Error> {
        Ok(*self.0.start()..=*self.0.end())
    }

    /// **NOTE:** This function should not be called with LstToSolRetVal
    fn sol_to_lst(&self, _lamports_amount: u64) -> Result<RangeInclusive<u64>, Self::Error> {
        Err(ProgramError(CUSTOM_ZERO))
    }
}

impl SolValCalc for SolToLstRetVal {
    type Error = ProgramError;

    fn lst_to_sol(&self, _lst_amount: u64) -> Result<RangeInclusive<u64>, Self::Error> {
        Err(ProgramError(CUSTOM_ZERO))
    }

    fn sol_to_lst(&self, _lamports_amount: u64) -> Result<RangeInclusive<u64>, Self::Error> {
        Ok(*self.0.start()..=*self.0.end())
    }
}

/// Wrapper for the return value from CPI call to `pricing` program
///
/// This is used to implement the `PriceExactIn` and `PriceExactOut` traits
/// so as to have reuse the same `quote_*` functions
pub struct PricingRetVal(pub u64);

impl PriceExactIn for PricingRetVal {
    type Error = ProgramError;

    fn price_exact_in(&self, _input: PriceExactInIxArgs) -> Result<u64, Self::Error> {
        Ok(self.0)
    }
}

impl PriceExactOut for PricingRetVal {
    type Error = ProgramError;

    fn price_exact_out(&self, _output: PriceExactOutIxArgs) -> Result<u64, Self::Error> {
        Ok(self.0)
    }
}
