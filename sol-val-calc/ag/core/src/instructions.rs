use inf1_ctl_core::svc::InfDummyCalcAccs;
use inf1_svc_core::traits::SolValCalcAccs;
use inf1_svc_lido_core::instructions::sol_val_calc::LidoCalcAccs;
use inf1_svc_marinade_core::instructions::sol_val_calc::MarinadeCalcAccs;
use inf1_svc_spl_core::instructions::sol_val_calc::{
    SanctumSplCalcAccs, SanctumSplMultiCalcAccs, SplCalcAccs,
};
use inf1_svc_wsol_core::instructions::sol_val_calc::WsolCalcAccs;

use crate::{map_variant_method, SvcAg};

pub type SvcCalcAccsAgRef<'a> = SvcAg<
    &'a InfDummyCalcAccs,
    &'a LidoCalcAccs,
    &'a MarinadeCalcAccs,
    &'a SanctumSplCalcAccs,
    &'a SanctumSplMultiCalcAccs,
    &'a SplCalcAccs,
    &'a WsolCalcAccs,
>;

pub type SvcCalcAccsAg = SvcAg<
    InfDummyCalcAccs,
    LidoCalcAccs,
    MarinadeCalcAccs,
    SanctumSplCalcAccs,
    SanctumSplMultiCalcAccs,
    SplCalcAccs,
    WsolCalcAccs,
>;

type InfKeysOwned = <InfDummyCalcAccs as SolValCalcAccs>::KeysOwned;
type InfAccFlags = <InfDummyCalcAccs as SolValCalcAccs>::AccFlags;

type LidoKeysOwned = <LidoCalcAccs as SolValCalcAccs>::KeysOwned;
type LidoAccFlags = <LidoCalcAccs as SolValCalcAccs>::AccFlags;

type MarinadeKeysOwned = <MarinadeCalcAccs as SolValCalcAccs>::KeysOwned;
type MarinadeAccFlags = <MarinadeCalcAccs as SolValCalcAccs>::AccFlags;

type SanctumSplKeysOwned = <SanctumSplCalcAccs as SolValCalcAccs>::KeysOwned;
type SanctumSplAccFlags = <SanctumSplCalcAccs as SolValCalcAccs>::AccFlags;

type SanctumSplMultiKeysOwned = <SanctumSplMultiCalcAccs as SolValCalcAccs>::KeysOwned;
type SanctumSplMultiAccFlags = <SanctumSplMultiCalcAccs as SolValCalcAccs>::AccFlags;

type SplKeysOwned = <SplCalcAccs as SolValCalcAccs>::KeysOwned;
type SplAccFlags = <SplCalcAccs as SolValCalcAccs>::AccFlags;

type WsolKeysOwned = <WsolCalcAccs as SolValCalcAccs>::KeysOwned;
type WsolAccFlags = <WsolCalcAccs as SolValCalcAccs>::AccFlags;

pub type SvcCalcAccsAgKeysOwned = SvcAg<
    InfKeysOwned,
    LidoKeysOwned,
    MarinadeKeysOwned,
    SanctumSplKeysOwned,
    SanctumSplMultiKeysOwned,
    SplKeysOwned,
    WsolKeysOwned,
>;

pub type SvcCalcAccsAgAccFlags = SvcAg<
    InfAccFlags,
    LidoAccFlags,
    MarinadeAccFlags,
    SanctumSplAccFlags,
    SanctumSplMultiAccFlags,
    SplAccFlags,
    WsolAccFlags,
>;

impl SvcCalcAccsAgRef<'_> {
    #[inline]
    pub const fn svc_suf_keys_owned(&self) -> SvcCalcAccsAgKeysOwned {
        map_variant_method!(self, svc_suf_keys_owned())
    }

    #[inline]
    pub const fn svc_suf_is_writer(&self) -> SvcCalcAccsAgAccFlags {
        map_variant_method!(self, svc_suf_is_writer())
    }

    #[inline]
    pub const fn svc_suf_is_signer(&self) -> SvcCalcAccsAgAccFlags {
        map_variant_method!(self, svc_suf_is_signer())
    }
}

impl SolValCalcAccs for SvcCalcAccsAgRef<'_> {
    type KeysOwned = SvcCalcAccsAgKeysOwned;
    type AccFlags = SvcCalcAccsAgAccFlags;

    #[inline]
    fn suf_keys_owned(&self) -> Self::KeysOwned {
        self.svc_suf_keys_owned()
    }

    #[inline]
    fn suf_is_writer(&self) -> Self::AccFlags {
        self.svc_suf_is_writer()
    }

    #[inline]
    fn suf_is_signer(&self) -> Self::AccFlags {
        self.svc_suf_is_signer()
    }
}

impl SolValCalcAccs for SvcCalcAccsAg {
    type KeysOwned = SvcCalcAccsAgKeysOwned;
    type AccFlags = SvcCalcAccsAgAccFlags;

    #[inline]
    fn suf_keys_owned(&self) -> Self::KeysOwned {
        self.as_ref_const().svc_suf_keys_owned()
    }

    #[inline]
    fn suf_is_writer(&self) -> Self::AccFlags {
        self.as_ref_const().svc_suf_is_writer()
    }

    #[inline]
    fn suf_is_signer(&self) -> Self::AccFlags {
        self.as_ref_const().svc_suf_is_signer()
    }
}
