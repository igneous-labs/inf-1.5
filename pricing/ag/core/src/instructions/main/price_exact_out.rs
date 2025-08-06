use inf1_pp_core::traits::main::PriceExactOutAccs;
use inf1_pp_flatfee_core::instructions::pricing::price::FlatFeePriceAccs;

use crate::PricingAg;

pub type PriceExactOutAccsAg = PricingAg<FlatFeePriceAccs>;

type FlatFeeKeysOwned = <FlatFeePriceAccs as PriceExactOutAccs>::KeysOwned;
type FlatFeeAccFlags = <FlatFeePriceAccs as PriceExactOutAccs>::AccFlags;

impl PriceExactOutAccs for PriceExactOutAccsAg {
    type KeysOwned = PricingAg<FlatFeeKeysOwned>;
    type AccFlags = PricingAg<FlatFeeAccFlags>;

    #[inline]
    fn suf_keys_owned(&self) -> Self::KeysOwned {
        match self {
            Self::FlatFee(p) => PricingAg::FlatFee(p.suf_keys_owned()),
        }
    }

    #[inline]
    fn suf_is_writer(&self) -> Self::AccFlags {
        match self {
            Self::FlatFee(p) => PricingAg::FlatFee(p.suf_is_writer()),
        }
    }

    #[inline]
    fn suf_is_signer(&self) -> Self::AccFlags {
        match self {
            Self::FlatFee(p) => PricingAg::FlatFee(p.suf_is_signer()),
        }
    }
}
