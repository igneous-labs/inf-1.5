use std::{
    error::Error,
    fmt::{self, Debug, Display, Formatter},
};

use inf1_std::{
    err::{InfErr, NotEnoughLiquidityErr},
    inf1_pp_ag_std::{
        inf1_pp_flatfee_std::{traits::FlatFeePricingColErr, update::FlatFeePricingUpdateErr},
        inf1_pp_flatslab_std::{
            traits::FlatSlabPricingColErr, typedefs::MintNotFoundErr,
            update::FlatSlabPricingUpdateErr,
        },
        pricing::PricingAgErr,
        PricingAg, PricingProgAgErr,
    },
    inf1_svc_ag_std::{
        calc::SvcCalcAgErr,
        update::{LidoUpdateErr, MarinadeUpdateErr, SplUpdateErr, UpdateSvcErr},
        SvcAg,
    },
    quote::{rebalance::RebalanceQuoteErr, swap::err::SwapQuoteErr},
    update::UpdateErr,
};
use solana_pubkey::Pubkey;

#[allow(deprecated)]
use inf1_std::quote::liquidity::remove::RemoveLiqQuoteErr;

/// Newtype wrapper to enable pretty-printing of pubkeys
#[repr(transparent)]
pub struct FmtErr<E>(pub E);

impl<E: Debug> Debug for FmtErr<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Display for FmtErr<InfErr> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.0 {
            InfErr::AccDeser { pk } => {
                f.write_fmt(format_args!("AccDeser: {}", Pubkey::new_from_array(pk)))
            }
            InfErr::MissingAcc { pk } => {
                f.write_fmt(format_args!("MissingAcc: {}", Pubkey::new_from_array(pk)))
            }
            InfErr::MissingSplData { mint } => f.write_fmt(format_args!(
                "MissingSplData: {}",
                Pubkey::new_from_array(mint)
            )),
            InfErr::MissingSvcData { mint } => f.write_fmt(format_args!(
                "MissingSvcData: {}",
                Pubkey::new_from_array(mint)
            )),
            InfErr::UnknownPp { pp_prog_id } => f.write_fmt(format_args!(
                "UnknownPp: {}",
                Pubkey::new_from_array(pp_prog_id)
            )),
            InfErr::UnknownSvc { svc_prog_id } => f.write_fmt(format_args!(
                "UnknownSvc: {}",
                Pubkey::new_from_array(svc_prog_id)
            )),
            InfErr::UnsupportedMint { mint } => f.write_fmt(format_args!(
                "UnsupportedMint: {}",
                Pubkey::new_from_array(mint)
            )),

            // inner wrapper
            InfErr::PricingProg(e) => Display::fmt(&FmtErr(e), f),
            InfErr::RebalanceQuote(e) => Display::fmt(&FmtErr(e), f),
            InfErr::RemoveLiqQuote(e) => Display::fmt(&FmtErr(e), f),
            InfErr::SwapQuote(e) => Display::fmt(&FmtErr(e), f),
            InfErr::UpdatePp(e) => Display::fmt(&FmtErr(e), f),
            InfErr::UpdateSvc(e) => Display::fmt(&FmtErr(e), f),

            // no need to wrap, no pubkey fields
            InfErr::AddLiqQuote(e) => Display::fmt(&e, f),

            // no special formatting
            InfErr::NoValidPda => Display::fmt(&self.0, f),
        }
    }
}

impl Error for FmtErr<InfErr> {}

impl Display for FmtErr<UpdateErr<InfErr>> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.0 {
            UpdateErr::AccMissing { pk } => {
                f.write_fmt(format_args!("MissingAcc: {}", Pubkey::new_from_array(pk)))
            }
            UpdateErr::Inner(_) => Display::fmt(&self.0, f),
        }
    }
}

impl Error for FmtErr<UpdateErr<InfErr>> {}

impl Display for FmtErr<PricingProgAgErr> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.0 {
            PricingAg::FlatFee(e) => Display::fmt(&FmtErr(e), f),
            PricingAg::FlatSlab(e) => Display::fmt(&FmtErr(e), f),
        }
    }
}

impl Display for FmtErr<FlatFeePricingColErr> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.0 {
            FlatFeePricingColErr::FeeAccountMissing { mint } => f.write_fmt(format_args!(
                "FeeAccountMissing: {}",
                Pubkey::new_from_array(mint)
            )),
            FlatFeePricingColErr::ProgramStateMissing => Display::fmt(&self.0, f),
        }
    }
}

impl Display for FmtErr<FlatSlabPricingColErr> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.0 {
            FlatSlabPricingColErr::MintNotFound(MintNotFoundErr { mint, .. }) => f.write_fmt(
                format_args!("MintNotFound: {}", Pubkey::new_from_array(mint)),
            ),
        }
    }
}

impl Display for FmtErr<RebalanceQuoteErr<SvcCalcAgErr, SvcCalcAgErr>> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.0 {
            RebalanceQuoteErr::NotEnoughLiquidity(e) => Display::fmt(&FmtErr(e), f),
            // all variants here dont have any fields that require formatting
            RebalanceQuoteErr::InpCalc(_)
            | RebalanceQuoteErr::OutCalc(_)
            | RebalanceQuoteErr::Overflow => Display::fmt(&self.0, f),
        }
    }
}

#[allow(deprecated)]
impl Display for FmtErr<RemoveLiqQuoteErr<SvcCalcAgErr, PricingAgErr>> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.0 {
            RemoveLiqQuoteErr::NotEnoughLiquidity(e) => Display::fmt(&FmtErr(e), f),
            // all variants here dont have any fields that require formatting
            RemoveLiqQuoteErr::OutCalc(_)
            | RemoveLiqQuoteErr::Pricing(_)
            | RemoveLiqQuoteErr::Overflow
            | RemoveLiqQuoteErr::ZeroValue => Display::fmt(&self.0, f),
        }
    }
}

impl Display for FmtErr<SwapQuoteErr<SvcCalcAgErr, SvcCalcAgErr, PricingAgErr>> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.0 {
            SwapQuoteErr::NotEnoughLiquidity(e) => Display::fmt(&FmtErr(e), f),
            // all variants here dont have any fields that require formatting
            SwapQuoteErr::InpCalc(_)
            | SwapQuoteErr::OutCalc(_)
            | SwapQuoteErr::Overflow
            | SwapQuoteErr::Pricing(_)
            | SwapQuoteErr::ZeroValue => Display::fmt(&self.0, f),
        }
    }
}

impl Display for FmtErr<NotEnoughLiquidityErr> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "NotEnoughLiquidity. Required: {}. Available: {}",
            self.0.required, self.0.available
        ))
    }
}

impl Display for FmtErr<PricingAg<FlatFeePricingUpdateErr, FlatSlabPricingUpdateErr>> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.0 {
            PricingAg::FlatFee(e) => Display::fmt(&FmtErr(e), f),
            PricingAg::FlatSlab(e) => Display::fmt(&FmtErr(e), f),
        }
    }
}

impl Display for FmtErr<FlatFeePricingUpdateErr> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.0 {
            FlatFeePricingUpdateErr::AccDeser { pk } => {
                f.write_fmt(format_args!("AccDeser: {}", Pubkey::new_from_array(pk)))
            }
        }
    }
}

impl Display for FmtErr<FlatSlabPricingUpdateErr> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.0 {
            FlatSlabPricingUpdateErr::AccDeser { pk } => {
                f.write_fmt(format_args!("AccDeser: {}", Pubkey::new_from_array(pk)))
            }
        }
    }
}

impl Display for FmtErr<UpdateSvcErr> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.0 {
            SvcAg::Lido(e) => Display::fmt(&FmtErr(e), f),
            SvcAg::Marinade(e) => Display::fmt(&FmtErr(e), f),
            SvcAg::SanctumSpl(e) | SvcAg::SanctumSplMulti(e) | SvcAg::Spl(e) => {
                Display::fmt(&FmtErr(e), f)
            }
            SvcAg::Wsol(_infallible) => unreachable!(),
        }
    }
}

impl Display for FmtErr<LidoUpdateErr> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.0 {
            LidoUpdateErr::AccDeser { pk } => {
                f.write_fmt(format_args!("AccDeser: {}", Pubkey::new_from_array(pk)))
            }
        }
    }
}

impl Display for FmtErr<MarinadeUpdateErr> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.0 {
            MarinadeUpdateErr::AccDeser { pk } => {
                f.write_fmt(format_args!("AccDeser: {}", Pubkey::new_from_array(pk)))
            }
        }
    }
}

impl Display for FmtErr<SplUpdateErr> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.0 {
            SplUpdateErr::AccDeser { pk } => {
                f.write_fmt(format_args!("AccDeser: {}", Pubkey::new_from_array(pk)))
            }
        }
    }
}
