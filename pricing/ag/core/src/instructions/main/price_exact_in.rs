use inf1_pp_core::traits::main::PriceExactInAccs;
use inf1_pp_flatfee_core::instructions::pricing::price::FlatFeePriceAccs;

use crate::PricingAg;

pub type PriceExactInAccsAg = PricingAg<FlatFeePriceAccs>;

type FlatFeeKeysOwned = <FlatFeePriceAccs as PriceExactInAccs>::KeysOwned;
type FlatFeeAccFlags = <FlatFeePriceAccs as PriceExactInAccs>::AccFlags;

impl PriceExactInAccs for PriceExactInAccsAg {
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
