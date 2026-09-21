use inf1_pp_core::traits::main::PriceExactOutAccs;
use inf1_pp_flatfee_core::instructions::pricing::price::FlatFeePriceAccs;
use inf1_pp_flatslab_core::instructions::pricing::FlatSlabPpAccs;
use inf1_pp_reserve_v2_core::instructions::pricing::ReserveV2PpAccs;

use crate::{internal_utils::map_variant, PricingAg};

pub type PriceExactOutAccsAg = PricingAg<FlatFeePriceAccs, FlatSlabPpAccs, ReserveV2PpAccs>;

type FlatFeeKeysOwned = <FlatFeePriceAccs as PriceExactOutAccs>::KeysOwned;
type FlatFeeAccFlags = <FlatFeePriceAccs as PriceExactOutAccs>::AccFlags;

type FlatSlabKeysOwned = <FlatSlabPpAccs as PriceExactOutAccs>::KeysOwned;
type FlatSlabAccFlags = <FlatSlabPpAccs as PriceExactOutAccs>::AccFlags;

type ReserveV2KeysOwned = <ReserveV2PpAccs as PriceExactOutAccs>::KeysOwned;
type ReserveV2AccFlags = <ReserveV2PpAccs as PriceExactOutAccs>::AccFlags;

impl PriceExactOutAccs for PriceExactOutAccsAg {
    type KeysOwned = PricingAg<FlatFeeKeysOwned, FlatSlabKeysOwned, ReserveV2KeysOwned>;
    type AccFlags = PricingAg<FlatFeeAccFlags, FlatSlabAccFlags, ReserveV2AccFlags>;

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
