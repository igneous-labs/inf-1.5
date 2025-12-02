use inf1_ctl_jiminy::instructions::swap::v2::IxPreKeysOwned;
use inf1_std::instructions::swap::{IxAccs, IxArgs};
use inf1_svc_ag_core::instructions::SvcCalcAccsAg;

mod exact_out;

type Accs<P> = IxAccs<[u8; 32], IxPreKeysOwned, SvcCalcAccsAg, SvcCalcAccsAg, P>;
type Args<P> = IxArgs<[u8; 32], IxPreKeysOwned, SvcCalcAccsAg, SvcCalcAccsAg, P>;
