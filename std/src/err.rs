#![allow(deprecated)]

use std::{error::Error, fmt::Display};

use inf1_core::quote::{
    liquidity::{add::AddLiqQuoteErr, remove::RemoveLiqQuoteErr},
    swap::err::SwapQuoteErr,
};
use inf1_pp_ag_std::PricingProgAgErr;
use inf1_svc_ag_std::calc::SvcCalcAgErr;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum InfErr {
    AccDeser { pk: [u8; 32] },
    AddLiqQuote(AddLiqQuoteErr<SvcCalcAgErr, PricingProgAgErr>),
    MissingAcc { pk: [u8; 32] },
    MissingSplData { mint: [u8; 32] },
    MissingSvcData { mint: [u8; 32] },
    NoValidPdaErr,
    Overflow,
    PoolErr,
    RemoveLiqQuote(RemoveLiqQuoteErr<SvcCalcAgErr, PricingProgAgErr>),
    SwapQuote(SwapQuoteErr<SvcCalcAgErr, SvcCalcAgErr, PricingProgAgErr>),
    UnknownPpErr { pp_prog_id: [u8; 32] },
    UnknownSvcErr { svc_prog_id: [u8; 32] },
    UnsupportedMintErr { mint: [u8; 32] },
}

impl Display for InfErr {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            InfErr::AccDeser { .. } => "AccDeser",
            InfErr::AddLiqQuote(..) => "AddLiqQuote",
            InfErr::MissingAcc { .. } => "MissingAcc",
            InfErr::MissingSplData { .. } => "MissingSplData",
            InfErr::MissingSvcData { .. } => "MissingSvcData",
            InfErr::NoValidPdaErr => "NoValidPdaErr",
            InfErr::Overflow => "Overflow",
            InfErr::PoolErr => "PoolErr",
            InfErr::RemoveLiqQuote(..) => "RemoveLiqQuote",
            InfErr::SwapQuote(..) => "SwapQuote",
            InfErr::UnknownPpErr { .. } => "UnknownPpErr",
            InfErr::UnknownSvcErr { .. } => "UnknownSvcErr",
            InfErr::UnsupportedMintErr { .. } => "UnsupportedMintErr",
        })
    }
}

impl Error for InfErr {}
