#![allow(deprecated)]

use std::{error::Error, fmt::Display};

use inf1_core::quote::{
    liquidity::{add::AddLiqQuoteErr, remove::RemoveLiqQuoteErr},
    swap::err::SwapQuoteErr,
};
use inf1_pp_ag_std::{update::UpdatePpErr, PricingProgAgErr};
use inf1_svc_ag_std::{calc::SvcCalcAgErr, update::UpdateSvcErr};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum InfErr {
    AccDeser { pk: [u8; 32] },
    AddLiqQuote(AddLiqQuoteErr<SvcCalcAgErr, PricingProgAgErr>),
    MissingAcc { pk: [u8; 32] },
    MissingSplData { mint: [u8; 32] },
    MissingSvcData { mint: [u8; 32] },
    NoValidPda,
    Overflow,
    PoolErr,
    RemoveLiqQuote(RemoveLiqQuoteErr<SvcCalcAgErr, PricingProgAgErr>),
    SwapQuote(SwapQuoteErr<SvcCalcAgErr, SvcCalcAgErr, PricingProgAgErr>),
    UnknownPp { pp_prog_id: [u8; 32] },
    UnknownSvc { svc_prog_id: [u8; 32] },
    UnsupportedMint { mint: [u8; 32] },
    UpdatePp(UpdatePpErr),
    UpdateSvc(UpdateSvcErr),
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
            InfErr::NoValidPda => "NoValidPdaErr",
            InfErr::Overflow => "Overflow",
            InfErr::PoolErr => "PoolErr",
            InfErr::RemoveLiqQuote(..) => "RemoveLiqQuote",
            InfErr::SwapQuote(..) => "SwapQuote",
            InfErr::UnknownPp { .. } => "UnknownPpErr",
            InfErr::UnknownSvc { .. } => "UnknownSvcErr",
            InfErr::UnsupportedMint { .. } => "UnsupportedMintErr",
            InfErr::UpdatePp { .. } => "UpdatePp",
            InfErr::UpdateSvc(..) => "UpdateSvc",
        })
    }
}

impl Error for InfErr {}
