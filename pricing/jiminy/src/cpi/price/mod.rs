use inf1_pp_core::instructions::price::IxAccs;
use jiminy_cpi::account::AccountHandle;

pub type IxAccountHandles<'a, P> = IxAccs<AccountHandle<'a>, P>;

/// `P: AsRef<[AccountHandle]>`
/// -> use [`PriceExactInIxAccountHandles::seq`] with [`jiminy_cpi::Cpi::invoke_fwd`]
pub type PriceExactInIxAccountHandles<'a, P> = IxAccountHandles<'a, P>;

/// `P: AsRef<[AccountHandle]>`
/// -> use [`PriceExactOutIxAccountHandles::seq`] with [`jiminy_cpi::Cpi::invoke_fwd`]
pub type PriceExactOutIxAccountHandles<'a, P> = IxAccountHandles<'a, P>;
