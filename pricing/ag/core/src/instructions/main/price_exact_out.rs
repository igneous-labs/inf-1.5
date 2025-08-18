use inf1_pp_core::traits::main::PriceExactOutAccs;
use inf1_pp_flatfee_core::instructions::pricing::price::FlatFeePriceAccs;
use inf1_pp_flatslab_core::instructions::pricing::FlatSlabPpAccs;

use crate::PricingAg;

pub type PriceExactOutAccsAg = PricingAg<FlatFeePriceAccs, FlatSlabPpAccs>;

type FlatFeeKeysOwned = <FlatFeePriceAccs as PriceExactOutAccs>::KeysOwned;
type FlatFeeAccFlags = <FlatFeePriceAccs as PriceExactOutAccs>::AccFlags;

type FlatSlabKeysOwned = <FlatSlabPpAccs as PriceExactOutAccs>::KeysOwned;
type FlatSlabAccFlags = <FlatSlabPpAccs as PriceExactOutAccs>::AccFlags;

impl PriceExactOutAccs for PriceExactOutAccsAg {
    type KeysOwned = PricingAg<FlatFeeKeysOwned, FlatSlabKeysOwned>;
    type AccFlags = PricingAg<FlatFeeAccFlags, FlatSlabAccFlags>;

    #[inline]
    fn suf_keys_owned(&self) -> Self::KeysOwned {
        match self {
            Self::FlatFee(p) => PricingAg::FlatFee(p.suf_keys_owned()),
            Self::FlatSlab(p) => PricingAg::FlatSlab(p.suf_keys_owned()),
        }
    }

    #[inline]
    fn suf_is_writer(&self) -> Self::AccFlags {
        match self {
            Self::FlatFee(p) => PricingAg::FlatFee(p.suf_is_writer()),
            Self::FlatSlab(p) => PricingAg::FlatSlab(p.suf_is_writer()),
        }
    }

    #[inline]
    fn suf_is_signer(&self) -> Self::AccFlags {
        match self {
            Self::FlatFee(p) => PricingAg::FlatFee(p.suf_is_signer()),
            Self::FlatSlab(p) => PricingAg::FlatSlab(p.suf_is_signer()),
        }
    }
}
