use inf1_ctl_jiminy::instructions::swap;
use inf1_std::instructions::swap::{IxAccs, IxArgs};
use inf1_svc_ag_core::instructions::SvcCalcAccsAg;

mod common;
mod v1;
mod v2;

type V1Accs<P> = IxAccs<[u8; 32], swap::v1::IxPreKeysOwned, SvcCalcAccsAg, SvcCalcAccsAg, P>;
type V1Args<P> = IxArgs<[u8; 32], swap::v1::IxPreKeysOwned, SvcCalcAccsAg, SvcCalcAccsAg, P>;

type V2Accs<P> = IxAccs<[u8; 32], swap::v2::IxPreKeysOwned, SvcCalcAccsAg, SvcCalcAccsAg, P>;
type V2Args<P> = IxArgs<[u8; 32], swap::v2::IxPreKeysOwned, SvcCalcAccsAg, SvcCalcAccsAg, P>;
