use inf1_ctl_jiminy::instructions::{
    liquidity as ctl_liq,
    swap::{v1 as ctl_v1, v2 as ctl_v2},
};
use inf1_pp_flatslab_std::instructions::pricing::FlatSlabPpAccs;
use inf1_std::instructions::{liquidity, swap};
use inf1_svc_ag_core::instructions::SvcCalcAccsAg;

mod common;
mod v1;
mod v2;

type V1Accs<P> = swap::IxAccs<[u8; 32], ctl_v1::IxPreKeysOwned, SvcCalcAccsAg, SvcCalcAccsAg, P>;
type V1Args<P> = swap::IxArgs<[u8; 32], ctl_v1::IxPreKeysOwned, SvcCalcAccsAg, SvcCalcAccsAg, P>;

type V2Accs<P> = swap::IxAccs<[u8; 32], ctl_v2::IxPreKeysOwned, SvcCalcAccsAg, SvcCalcAccsAg, P>;
type V2Args<P> = swap::IxArgs<[u8; 32], ctl_v2::IxPreKeysOwned, SvcCalcAccsAg, SvcCalcAccsAg, P>;

// only flatslab has uniform interface across all 4 pp instructions
type LiqAccs = liquidity::IxAccs<[u8; 32], ctl_liq::IxPreKeysOwned, SvcCalcAccsAg, FlatSlabPpAccs>;
type LiqArgs = liquidity::IxArgs<[u8; 32], ctl_liq::IxPreKeysOwned, SvcCalcAccsAg, FlatSlabPpAccs>;
