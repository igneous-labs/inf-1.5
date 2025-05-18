use inf1_svc_core::traits::SolValCalcAccs;
use inf1_svc_generic::instructions::{
    IxSufAccFlags as GenericSufAccFlags, IxSufKeysOwned as GenericSufKeysOwned,
};
use inf1_svc_lido_core::instructions::sol_val_calc::LidoCalcAccs;
use inf1_svc_marinade_core::instructions::sol_val_calc::MarinadeCalcAccs;
use inf1_svc_spl_core::instructions::sol_val_calc::{
    SanctumSplCalcAccs, SanctumSplMultiCalcAccs, SplCalcAccs,
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
