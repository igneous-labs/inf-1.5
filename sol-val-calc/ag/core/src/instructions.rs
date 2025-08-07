use core::iter::{empty, once};

use inf1_svc_core::traits::SolValCalcAccs;
use inf1_svc_lido_core::{
    instructions::sol_val_calc::LidoCalcAccs, solido_legacy_core::LIDO_STATE_ADDR,
};
use inf1_svc_marinade_core::instructions::sol_val_calc::MarinadeCalcAccs;
use inf1_svc_spl_core::{
    instructions::sol_val_calc::{SanctumSplCalcAccs, SanctumSplMultiCalcAccs, SplCalcAccs},
    sanctum_spl_stake_pool_core::SYSVAR_CLOCK,
};
use inf1_svc_wsol_core::instructions::sol_val_calc::WsolCalcAccs;

use crate::SvcAg;

pub type SvcCalcAccsAg = SvcAg<
    LidoCalcAccs,
    MarinadeCalcAccs,
    SanctumSplCalcAccs,
    SanctumSplMultiCalcAccs,
    SplCalcAccs,
    WsolCalcAccs,
>;

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
    LidoKeysOwned,
    MarinadeKeysOwned,
    SanctumSplKeysOwned,
    SanctumSplMultiKeysOwned,
    SplKeysOwned,
    WsolKeysOwned,
>;

pub type SvcCalcAccsAgAccFlags = SvcAg<
    LidoAccFlags,
    MarinadeAccFlags,
    SanctumSplAccFlags,
    SanctumSplMultiAccFlags,
    SplAccFlags,
    WsolAccFlags,
>;

impl SvcCalcAccsAg {
    #[inline]
    pub const fn svc_suf_keys_owned(&self) -> SvcCalcAccsAgKeysOwned {
        match self {
            Self::Lido(c) => SvcCalcAccsAgKeysOwned::Lido(c.svc_suf_keys_owned()),
            Self::Marinade(c) => SvcCalcAccsAgKeysOwned::Marinade(c.svc_suf_keys_owned()),
            Self::SanctumSpl(c) => SvcCalcAccsAgKeysOwned::SanctumSpl(c.svc_suf_keys_owned()),
            Self::SanctumSplMulti(c) => {
                SvcCalcAccsAgKeysOwned::SanctumSplMulti(c.svc_suf_keys_owned())
            }
            Self::Spl(c) => SvcCalcAccsAgKeysOwned::Spl(c.svc_suf_keys_owned()),
            Self::Wsol(c) => SvcCalcAccsAgKeysOwned::Wsol(c.svc_suf_keys_owned()),
        }
    }

    #[inline]
    pub const fn svc_suf_is_writer(&self) -> SvcCalcAccsAgAccFlags {
        match self {
            Self::Lido(c) => SvcCalcAccsAgAccFlags::Lido(c.svc_suf_is_writer()),
            Self::Marinade(c) => SvcCalcAccsAgAccFlags::Marinade(c.svc_suf_is_writer()),
            Self::SanctumSpl(c) => SvcCalcAccsAgAccFlags::SanctumSpl(c.svc_suf_is_writer()),
            Self::SanctumSplMulti(c) => {
                SvcCalcAccsAgAccFlags::SanctumSplMulti(c.svc_suf_is_writer())
            }
            Self::Spl(c) => SvcCalcAccsAgAccFlags::Spl(c.svc_suf_is_writer()),
            Self::Wsol(c) => SvcCalcAccsAgAccFlags::Wsol(c.svc_suf_is_writer()),
        }
    }

    #[inline]
    pub const fn svc_suf_is_signer(&self) -> SvcCalcAccsAgAccFlags {
        match self {
            Self::Lido(c) => SvcCalcAccsAgAccFlags::Lido(c.svc_suf_is_signer()),
            Self::Marinade(c) => SvcCalcAccsAgAccFlags::Marinade(c.svc_suf_is_signer()),
            Self::SanctumSpl(c) => SvcCalcAccsAgAccFlags::SanctumSpl(c.svc_suf_is_signer()),
            Self::SanctumSplMulti(c) => {
                SvcCalcAccsAgAccFlags::SanctumSplMulti(c.svc_suf_is_signer())
            }
            Self::Spl(c) => SvcCalcAccsAgAccFlags::Spl(c.svc_suf_is_signer()),
            Self::Wsol(c) => SvcCalcAccsAgAccFlags::Wsol(c.svc_suf_is_signer()),
        }
    }
}

impl SolValCalcAccs for SvcCalcAccsAg {
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

// TODO: deleteme and move this to inf1-svc-ag-std instead as part of update traits impl

type LidoPkIter = core::array::IntoIter<[u8; 32], 2>;
type MarinadePkIter = core::iter::Once<[u8; 32]>;
type SanctumSplPkIter = core::array::IntoIter<[u8; 32], 2>;
type SanctumSplMultiPkIter = core::array::IntoIter<[u8; 32], 2>;
type SplPkIter = core::array::IntoIter<[u8; 32], 2>;
type WsolPkIter = core::iter::Empty<[u8; 32]>;

pub type SvcPkIterAg = SvcAg<
    LidoPkIter,
    MarinadePkIter,
    SanctumSplPkIter,
    SanctumSplMultiPkIter,
    SplPkIter,
    WsolPkIter,
>;

impl SvcCalcAccsAg {
    /// Pubkey of the accounts from which the corresponding
    /// [`inf1_svc_core::traits::SolValCalc`] is derived from.
    ///
    /// These accounts should be fetched and deserialized to create the `SolValCalc`.
    #[inline]
    pub fn calc_keys(&self) -> SvcPkIterAg {
        match self {
            SvcAg::Lido(_) => SvcAg::Lido([LIDO_STATE_ADDR, SYSVAR_CLOCK].into_iter()),
            SvcAg::Marinade(_) => SvcAg::Marinade(once(
                inf1_svc_marinade_core::sanctum_marinade_liquid_staking_core::STATE_PUBKEY,
            )),
            SvcAg::SanctumSpl(SanctumSplCalcAccs { stake_pool_addr }) => {
                SvcAg::SanctumSpl([*stake_pool_addr, SYSVAR_CLOCK].into_iter())
            }
            SvcAg::SanctumSplMulti(SanctumSplMultiCalcAccs { stake_pool_addr }) => {
                SvcAg::SanctumSplMulti([*stake_pool_addr, SYSVAR_CLOCK].into_iter())
            }
            SvcAg::Spl(SplCalcAccs { stake_pool_addr }) => {
                SvcAg::Spl([*stake_pool_addr, SYSVAR_CLOCK].into_iter())
            }
            SvcAg::Wsol(_) => SvcAg::Wsol(empty()),
        }
    }
}
