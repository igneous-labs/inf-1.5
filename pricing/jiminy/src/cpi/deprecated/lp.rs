use inf1_pp_core::instructions::deprecated::lp::IxAccs;
use jiminy_cpi::account::AccountHandle;

pub type IxAccountHandles<'a, P> = IxAccs<AccountHandle<'a>, P>;

/// `P: AsRef<[AccountHandle]>`
/// -> use [`PriceLpTokensToMintIxAccountHandles::seq`] with [`jiminy_cpi::Cpi::invoke_fwd`]
pub type PriceLpTokensToMintIxAccountHandles<'a, P> = IxAccountHandles<'a, P>;

/// `P: AsRef<[AccountHandle]>`
/// -> use [`PriceLpTokensToRedeemIxAccountHandles::seq`] with [`jiminy_cpi::Cpi::invoke_fwd`]
pub type PriceLpTokensToRedeemIxAccountHandles<'a, P> = IxAccountHandles<'a, P>;
