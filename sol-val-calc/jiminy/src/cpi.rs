use inf1_svc_core::instructions::IxAccs;
use jiminy_cpi::account::AccountHandle;

pub type IxAccountHandles<'a, S> = IxAccs<AccountHandle<'a>, S>;

/// `S: AsRef<[AccountHandle]>`
/// -> use [`IxAccountHandles::seq`] with [`jiminy_cpi::Cpi::invoke_fwd`]
pub type SvcIxAccountHandles<'a, S> = IxAccountHandles<'a, S>;
