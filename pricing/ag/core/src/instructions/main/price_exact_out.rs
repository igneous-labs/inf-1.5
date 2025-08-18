use inf1_pp_core::traits::main::PriceExactOutAccs;
use inf1_pp_flatfee_core::instructions::pricing::price::FlatFeePriceAccs;
use inf1_pp_flatslab_core::instructions::pricing::FlatSlabPpAccs;

use crate::{internal_utils::map_variant, PricingAg};

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
        map_variant!(self, PriceExactOutAccs::suf_keys_owned)
    }

    #[inline]
    fn suf_is_writer(&self) -> Self::AccFlags {
        map_variant!(self, PriceExactOutAccs::suf_is_writer)
    }

    #[inline]
    fn suf_is_signer(&self) -> Self::AccFlags {
        map_variant!(self, PriceExactOutAccs::suf_is_signer)
    }
}
