use inf1_ctl_jiminy::instructions::{
    liquidity as ctl_liq,
    swap::{v1 as ctl_v1, v2 as ctl_v2},
};
use inf1_pp_ag_core::{
    inf1_pp_flatfee_core::{
        instructions::pricing::price::FlatFeePriceAccs, pricing::price::FlatFeeSwapPricing,
    },
    PricingAg,
};
use inf1_pp_flatslab_std::{instructions::pricing::FlatSlabPpAccs, pricing::FlatSlabSwapPricing};
use inf1_std::{
    instructions::{liquidity, swap},
    quote::swap::QuoteArgs,
};
use inf1_svc_ag_core::{calc::SvcCalcAg, instructions::SvcCalcAccsAg};

mod common;
mod v1;
mod v2;

/// impls both PriceExactInAccs and PriceExactOutAccs (but not deprectated LP interfaces)
type PricingSwapAccsAg = PricingAg<FlatFeePriceAccs, FlatSlabPpAccs>;

/// impls both PriceExactIn and PriceExactOut (but not deprectated LP interfaces)
type PricingSwapAg = PricingAg<FlatFeeSwapPricing, FlatSlabSwapPricing>;

type QuoteArgsAg = QuoteArgs<SvcCalcAg, SvcCalcAg, PricingSwapAg>;

type V1Accs =
    swap::IxAccs<[u8; 32], ctl_v1::IxPreKeysOwned, SvcCalcAccsAg, SvcCalcAccsAg, PricingSwapAccsAg>;
type V1Args =
    swap::IxArgs<[u8; 32], ctl_v1::IxPreKeysOwned, SvcCalcAccsAg, SvcCalcAccsAg, PricingSwapAccsAg>;

type V2Accs =
    swap::IxAccs<[u8; 32], ctl_v2::IxPreKeysOwned, SvcCalcAccsAg, SvcCalcAccsAg, PricingSwapAccsAg>;
type V2Args =
    swap::IxArgs<[u8; 32], ctl_v2::IxPreKeysOwned, SvcCalcAccsAg, SvcCalcAccsAg, PricingSwapAccsAg>;

// only flatslab has uniform interface across all 4 pp instructions
type LiqAccs = liquidity::IxAccs<[u8; 32], ctl_liq::IxPreKeysOwned, SvcCalcAccsAg, FlatSlabPpAccs>;
type LiqArgs = liquidity::IxArgs<[u8; 32], ctl_liq::IxPreKeysOwned, SvcCalcAccsAg, FlatSlabPpAccs>;
