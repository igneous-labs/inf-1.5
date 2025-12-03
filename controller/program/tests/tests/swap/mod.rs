use inf1_ctl_jiminy::instructions::swap::v2::IxPreKeysOwned;
use inf1_std::instructions::swap::{IxAccs, IxArgs};
use inf1_svc_ag_core::instructions::SvcCalcAccsAg;

mod common;
mod v2;

type Accs<P> = IxAccs<[u8; 32], IxPreKeysOwned, SvcCalcAccsAg, SvcCalcAccsAg, P>;
type Args<P> = IxArgs<[u8; 32], IxPreKeysOwned, SvcCalcAccsAg, SvcCalcAccsAg, P>;

// TODO: uncomment when fixed with v2
// mod v1
