use inf1_pp_core::traits::main::PriceExactOutAccs;
use inf1_pp_flatfee_core::instructions::pricing::price::FlatFeePriceAccs;

use crate::PricingAccsAg;

pub type PriceExactOutAccsAg = PricingAccsAg<FlatFeePriceAccs>;

type FlatFeeKeysOwned = <FlatFeePriceAccs as PriceExactOutAccs>::KeysOwned;
type FlatFeeAccFlags = <FlatFeePriceAccs as PriceExactOutAccs>::AccFlags;

impl PriceExactOutAccs for PriceExactOutAccsAg {
    type KeysOwned = PricingAccsAg<FlatFeeKeysOwned>;
    type AccFlags = PricingAccsAg<FlatFeeAccFlags>;

    #[inline]
    fn suf_keys_owned(&self) -> Self::KeysOwned {
        match self {
            Self::FlatFee(p) => PricingAccsAg::FlatFee(p.suf_keys_owned()),
        }
    }

    #[inline]
    fn suf_is_writer(&self) -> Self::AccFlags {
        match self {
            Self::FlatFee(p) => PricingAccsAg::FlatFee(p.suf_is_writer()),
        }
    }

    #[inline]
    fn suf_is_signer(&self) -> Self::AccFlags {
        match self {
            Self::FlatFee(p) => PricingAccsAg::FlatFee(p.suf_is_signer()),
        }
    }
}
