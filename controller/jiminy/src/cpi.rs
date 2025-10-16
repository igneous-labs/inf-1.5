use inf1_ctl_core::instructions::{
    liquidity::add::AddLiquidityIxPreAccs, sync_sol_value::SyncSolValueIxPreAccs,
};
use jiminy_cpi::account::AccountHandle;

/// `S: AsRef<[AccountHandle]>`
/// -> use [`IxAccountHandles::seq`] with [`jiminy_cpi::Cpi::invoke_fwd`]
pub type SyncSolValueIxPreAccountHandles<'account> = SyncSolValueIxPreAccs<AccountHandle<'account>>;
pub type AddLiquidityIxPreAccountHandles<'account> = AddLiquidityIxPreAccs<AccountHandle<'account>>;

// TODO: make invoke() helpers for client programs
