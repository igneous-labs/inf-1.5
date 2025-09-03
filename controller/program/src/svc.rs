use jiminy_cpi::account::AccountHandle;

pub type SvcIxAccountHandles<'a, 'acc> =
    inf1_svc_jiminy::cpi::SvcIxAccountHandles<'acc, &'a [AccountHandle<'acc>]>;

// rename to make disambiguate type name
/// Accounts builder for SolToLst and LstToSol
pub use inf1_svc_jiminy::instructions::NewIxPreAccsBuilder as NewSvcIxPreAccsBuilder;
