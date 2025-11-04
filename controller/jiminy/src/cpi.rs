use core::ops::RangeInclusive;
use inf1_ctl_core::instructions::{
    admin::set_sol_value_calculator::SetSolValueCalculatorIxPreAccs,
    rebalance::{end::EndRebalanceIxPreAccs, start::StartRebalanceIxPreAccs},
    swap::IxPreAccs as SwapIxPreAccs,
    sync_sol_value::SyncSolValueIxPreAccs,
};

use inf1_pp_core::{
    instructions::price::{exact_in::PriceExactInIxArgs, exact_out::PriceExactOutIxArgs},
    traits::main::{PriceExactIn, PriceExactOut},
};
use inf1_svc_core::traits::SolValCalc;
use jiminy_cpi::account::AccountHandle;
use jiminy_program_error::ProgramError;

/// `S: AsRef<[AccountHandle]>`
/// -> use [`IxAccountHandles::seq`] with [`jiminy_cpi::Cpi::invoke_fwd`]
pub type SyncSolValueIxPreAccountHandles<'account> = SyncSolValueIxPreAccs<AccountHandle<'account>>;

/// `S: AsRef<[AccountHandle]>`
/// -> use [`IxAccountHandles::seq`] with [`jiminy_cpi::Cpi::invoke_fwd`]
pub type SetSolValueCalculatorIxPreAccountHandles<'account> =
    SetSolValueCalculatorIxPreAccs<AccountHandle<'account>>;

/// `S: AsRef<[AccountHandle]>`
/// -> use [`IxAccountHandles::seq`] with [`jiminy_cpi::Cpi::invoke_fwd`]
pub type StartRebalanceIxPreAccountHandles<'account> =
    StartRebalanceIxPreAccs<AccountHandle<'account>>;

/// `S: AsRef<[AccountHandle]>`
/// -> use [`IxAccountHandles::seq`] with [`jiminy_cpi::Cpi::invoke_fwd`]
pub type EndRebalanceIxPreAccountHandles<'account> = EndRebalanceIxPreAccs<AccountHandle<'account>>;

pub type SwapIxPreAccountHandles<'account> = SwapIxPreAccs<AccountHandle<'account>>;

// TODO: make invoke() helpers for client programs

/// Wrapper for the return value from CPI call to `sol-val-calc` program
///
/// This is then used to implement the `SolValCalc` trait
/// so as to have re-use the same `quote_*` functions
#[repr(transparent)]
pub struct LstToSolRetVal(pub RangeInclusive<u64>);

/// Wrapper for the return value from CPI call to `sol-val-calc` program
///
/// This is then used to implement the `SolValCalc` trait
/// so as to have re-use the same `quote_*` functions
#[repr(transparent)]
pub struct SolToLstRetVal(pub RangeInclusive<u64>);

impl SolValCalc for LstToSolRetVal {
    type Error = ProgramError;

    fn lst_to_sol(&self, _lst_amount: u64) -> Result<RangeInclusive<u64>, Self::Error> {
        Ok(self.0.clone())
    }

    /// **NOTE:** This function should not be called with LstToSolRetVal
    fn sol_to_lst(&self, _lamports_amount: u64) -> Result<RangeInclusive<u64>, Self::Error> {
        unreachable!()
    }
}

impl SolValCalc for SolToLstRetVal {
    type Error = ProgramError;

    fn lst_to_sol(&self, _lst_amount: u64) -> Result<RangeInclusive<u64>, Self::Error> {
        unreachable!()
    }

    fn sol_to_lst(&self, _lamports_amount: u64) -> Result<RangeInclusive<u64>, Self::Error> {
        Ok(self.0.clone())
    }
}

/// Wrapper for the return value from CPI call to `pricing` program
///
/// This is used to implement the `PriceExactIn` and `PriceExactOut` traits
/// so as to have reuse the same `quote_*` functions
#[repr(transparent)]
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
