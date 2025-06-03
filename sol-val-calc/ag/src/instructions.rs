use inf1_svc_core::traits::SolValCalcAccs;
use inf1_svc_generic::instructions::{
    IxSufAccFlags as GenericSufAccFlags, IxSufKeysOwned as GenericSufKeysOwned,
};
use inf1_svc_lido_core::{
    instructions::sol_val_calc::LidoCalcAccs, solido_legacy_core::LIDO_STATE_ADDR,
};
use inf1_svc_marinade_core::instructions::sol_val_calc::MarinadeCalcAccs;
use inf1_svc_spl_core::{
    instructions::sol_val_calc::{SanctumSplCalcAccs, SanctumSplMultiCalcAccs, SplCalcAccs},
    sanctum_spl_stake_pool_core::SYSVAR_CLOCK,
};
use inf1_svc_wsol_core::instructions::sol_val_calc::{
    IxSufAccFlags as WsolSufAccsFlag, IxSufKeysOwned as WsolSufKeysOwned, WsolCalcAccs,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CalcAccsAg {
    Lido,
    Marinade,
    SanctumSpl(SanctumSplCalcAccs),
    SanctumSplMulti(SanctumSplMultiCalcAccs),
    Spl(SplCalcAccs),
    Wsol,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CalcAccsAgTy {
    Lido,
    Marinade,
    SanctumSpl,
    SanctumSplMulti,
    Spl,
    Wsol,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IxSufAccsAg<G, W> {
    Generic(G),
    Wsol(W),
}

impl<A, G, W> AsRef<A> for IxSufAccsAg<G, W>
where
    A: ?Sized,
    G: AsRef<A>,
    W: AsRef<A>,
{
    #[inline]
    fn as_ref(&self) -> &A {
        match self {
            Self::Generic(g) => g.as_ref(),
            Self::Wsol(w) => w.as_ref(),
        }
    }
}

pub type IxSufKeysOwnedAg = IxSufAccsAg<GenericSufKeysOwned, WsolSufKeysOwned>;

pub type IxSufAccFlagsAg = IxSufAccsAg<GenericSufAccFlags, WsolSufAccsFlag>;

impl CalcAccsAg {
    #[inline]
    pub const fn svc_suf_keys_owned(&self) -> IxSufKeysOwnedAg {
        match self {
            Self::Lido => IxSufKeysOwnedAg::Generic(LidoCalcAccs.svc_suf_keys_owned()),
            Self::Marinade => IxSufKeysOwnedAg::Generic(MarinadeCalcAccs.svc_suf_keys_owned()),
            Self::SanctumSpl(s) => IxSufKeysOwnedAg::Generic(s.svc_suf_keys_owned()),
            Self::SanctumSplMulti(s) => IxSufKeysOwnedAg::Generic(s.svc_suf_keys_owned()),
            Self::Spl(s) => IxSufKeysOwnedAg::Generic(s.svc_suf_keys_owned()),
            Self::Wsol => IxSufAccsAg::Wsol(WsolCalcAccs.svc_suf_keys_owned()),
        }
    }

    #[inline]
    pub const fn svc_suf_is_writer(&self) -> IxSufAccFlagsAg {
        match self {
            Self::Lido => IxSufAccFlagsAg::Generic(LidoCalcAccs.svc_suf_is_writer()),
            Self::Marinade => IxSufAccFlagsAg::Generic(MarinadeCalcAccs.svc_suf_is_writer()),
            Self::SanctumSpl(s) => IxSufAccFlagsAg::Generic(s.svc_suf_is_writer()),
            Self::SanctumSplMulti(s) => IxSufAccFlagsAg::Generic(s.svc_suf_is_writer()),
            Self::Spl(s) => IxSufAccFlagsAg::Generic(s.svc_suf_is_writer()),
            Self::Wsol => IxSufAccsAg::Wsol(WsolCalcAccs.svc_suf_is_writer()),
        }
    }

    #[inline]
    pub const fn svc_suf_is_signer(&self) -> IxSufAccFlagsAg {
        match self {
            Self::Lido => IxSufAccFlagsAg::Generic(LidoCalcAccs.svc_suf_is_signer()),
            Self::Marinade => IxSufAccFlagsAg::Generic(MarinadeCalcAccs.svc_suf_is_signer()),
            Self::SanctumSpl(s) => IxSufAccFlagsAg::Generic(s.svc_suf_is_signer()),
            Self::SanctumSplMulti(s) => IxSufAccFlagsAg::Generic(s.svc_suf_is_signer()),
            Self::Spl(s) => IxSufAccFlagsAg::Generic(s.svc_suf_is_signer()),
            Self::Wsol => IxSufAccsAg::Wsol(WsolCalcAccs.svc_suf_is_signer()),
        }
    }
}

impl SolValCalcAccs for CalcAccsAg {
    type KeysOwned = IxSufKeysOwnedAg;
    type AccFlags = IxSufAccFlagsAg;

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

/// Getters
impl CalcAccsAg {
    /// Pubkey of the accounts from which the corresponding
    /// [`inf1_svc_core::traits::SolValCalc`] is derived from.
    ///
    /// These accounts should be fetched and deserialized to create the `SolValCalc`.
    #[inline]
    pub fn calc_keys(&self) -> impl Iterator<Item = &[u8; 32]> {
        match self {
            CalcAccsAg::Lido => Some(&LIDO_STATE_ADDR).into_iter().chain(&[SYSVAR_CLOCK]),
            CalcAccsAg::Marinade => {
                Some(&inf1_svc_marinade_core::sanctum_marinade_liquid_staking_core::STATE_PUBKEY)
                    .into_iter()
                    .chain(&[])
            }
            CalcAccsAg::SanctumSpl(SanctumSplCalcAccs { stake_pool_addr })
            | CalcAccsAg::SanctumSplMulti(SanctumSplMultiCalcAccs { stake_pool_addr })
            | CalcAccsAg::Spl(SplCalcAccs { stake_pool_addr }) => {
                Some(stake_pool_addr).into_iter().chain(&[SYSVAR_CLOCK])
            }
            CalcAccsAg::Wsol => None.into_iter().chain(&[]),
        }
    }

    #[inline]
    pub const fn ty(&self) -> CalcAccsAgTy {
        match self {
            Self::Lido => CalcAccsAgTy::Lido,
            Self::Marinade => CalcAccsAgTy::Marinade,
            Self::SanctumSpl(_) => CalcAccsAgTy::SanctumSpl,
            Self::SanctumSplMulti(_) => CalcAccsAgTy::SanctumSplMulti,
            Self::Spl(_) => CalcAccsAgTy::Spl,
            Self::Wsol => CalcAccsAgTy::Wsol,
        }
    }
}

impl CalcAccsAgTy {
    #[inline]
    pub const fn program_id(&self) -> &[u8; 32] {
        match self {
            Self::Lido => &inf1_svc_lido_core::ID,
            Self::Marinade => &inf1_svc_marinade_core::ID,
            Self::SanctumSpl => &inf1_svc_spl_core::keys::sanctum_spl::ID,
            Self::SanctumSplMulti => &inf1_svc_spl_core::keys::sanctum_spl_multi::ID,
            Self::Spl => &inf1_svc_spl_core::keys::spl::ID,
            Self::Wsol => &inf1_svc_wsol_core::ID,
        }
    }

    #[inline]
    pub const fn try_from_program_id(program_id: &[u8; 32]) -> Option<Self> {
        Some(match program_id {
            &inf1_svc_lido_core::ID => Self::Lido,
            &inf1_svc_marinade_core::ID => Self::Marinade,
            &inf1_svc_spl_core::keys::sanctum_spl::ID => Self::SanctumSpl,
            &inf1_svc_spl_core::keys::sanctum_spl_multi::ID => Self::SanctumSplMulti,
            &inf1_svc_spl_core::keys::spl::ID => Self::Spl,
            &inf1_svc_wsol_core::ID => Self::Wsol,
            _ => return None,
        })
    }
}
