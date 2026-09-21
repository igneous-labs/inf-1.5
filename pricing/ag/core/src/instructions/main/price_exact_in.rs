use inf1_pp_core::traits::main::PriceExactInAccs;
use inf1_pp_flatfee_core::instructions::pricing::price::FlatFeePriceAccs;
use inf1_pp_flatslab_core::instructions::pricing::FlatSlabPpAccs;
use inf1_pp_reserve_v2_core::instructions::pricing::ReserveV2PpAccs;

use crate::{internal_utils::map_variant, PricingAg};

pub type PriceExactInAccsAg = PricingAg<FlatFeePriceAccs, FlatSlabPpAccs, ReserveV2PpAccs>;

type FlatFeeKeysOwned = <FlatFeePriceAccs as PriceExactInAccs>::KeysOwned;
type FlatFeeAccFlags = <FlatFeePriceAccs as PriceExactInAccs>::AccFlags;

type FlatSlabKeysOwned = <FlatSlabPpAccs as PriceExactInAccs>::KeysOwned;
type FlatSlabAccFlags = <FlatSlabPpAccs as PriceExactInAccs>::AccFlags;

type ReserveV2KeysOwned = <ReserveV2PpAccs as PriceExactInAccs>::KeysOwned;
type ReserveV2AccFlags = <ReserveV2PpAccs as PriceExactInAccs>::AccFlags;

impl PriceExactInAccs for PriceExactInAccsAg {
    type KeysOwned = PricingAg<FlatFeeKeysOwned, FlatSlabKeysOwned, ReserveV2KeysOwned>;
    type AccFlags = PricingAg<FlatFeeAccFlags, FlatSlabAccFlags, ReserveV2AccFlags>;

    #[inline]
    fn suf_keys_owned(&self) -> Self::KeysOwned {
        map_variant!(self, PriceExactInAccs::suf_keys_owned)
    }

    #[inline]
    fn suf_is_writer(&self) -> Self::AccFlags {
        map_variant!(self, PriceExactInAccs::suf_is_writer)
    }

    #[inline]
    fn suf_is_signer(&self) -> Self::AccFlags {
        map_variant!(self, PriceExactInAccs::suf_is_signer)
    }
}
