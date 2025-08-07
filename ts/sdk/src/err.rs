use std::convert::Infallible;

use bs58_fixed::Bs58String;
use inf1_core::{err::NotEnoughLiquidityErr, quote::swap::err::SwapQuoteErr};
use inf1_pp_ag_std::PricingAg;
use inf1_pp_flatfee_std::{
    pricing::err::FlatFeePricingErr, traits::FlatFeePricingColErr, update::FlatFeePricingUpdateErr,
};
use inf1_svc_ag_core::{
    inf1_svc_lido_core::calc::LidoCalcErr, inf1_svc_marinade_core::calc::MarinadeCalcErr,
    inf1_svc_spl_core::calc::SplCalcErr, SvcAg,
};
use inf1_update_traits::UpdateErr;
use serde::{Deserialize, Serialize};
use tsify_next::Tsify;
use wasm_bindgen::prelude::*;

#[allow(deprecated)]
use inf1_core::quote::liquidity::{add::AddLiqQuoteErr, remove::RemoveLiqQuoteErr};

type Bs58PkString = Bs58String<44>;

const ERR_CODE_MSG_SEP: &str = ":";

/// All {@link Error} objects thrown by SDK functions will start with
/// `{InfErr}:`, so that the `InfErr` error code can be
/// extracted by splitting on the first colon `:`
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[allow(clippy::enum_variant_names)] // we want all the ts consts to have `Err` suffix
pub enum InfErr {
    AccDeserErr,
    InternalErr,
    MissingAccErr,
    MissingSplDataErr,
    MissingSvcDataErr,
    NoValidPdaErr,
    PoolErr,
    UnknownPpErr,
    UnknownSvcErr,
    UnsupportedMintErr,
    UserErr,
}

/// Top level error, all fallible functions should
/// have this as Result's err type to throw the appropriate `JsError`
#[derive(Debug)]
pub struct InfError {
    pub code: InfErr,

    pub cause: Option<String>,
}

impl From<InfError> for JsValue {
    fn from(InfError { code, cause }: InfError) -> Self {
        let suf = cause.unwrap_or_default();
        JsError::new(&format!("{code:?}{ERR_CODE_MSG_SEP}{suf}")).into()
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct AllInfErrs(#[tsify(type = "InfErr[]")] pub [InfErr; 10]);

/// Returns the array of all possible {@link InfErr}s
#[wasm_bindgen(js_name = allInfErrs)]
pub fn all_inf_errs() -> AllInfErrs {
    use InfErr::*;

    AllInfErrs([
        AccDeserErr,
        InternalErr,
        MissingAccErr,
        MissingSplDataErr,
        MissingSvcDataErr,
        NoValidPdaErr,
        PoolErr,
        UnknownSvcErr,
        UnsupportedMintErr,
        UserErr,
    ])
}

pub(crate) fn acc_deser_err(pk: &[u8; 32]) -> InfError {
    let pk = Bs58PkString::encode(pk);
    InfError {
        code: InfErr::AccDeserErr,
        cause: Some(format!("account data for {pk} not of expected format",)),
    }
}

pub(crate) fn missing_acc_err(pk: &[u8; 32]) -> InfError {
    let pk = Bs58PkString::encode(pk);
    InfError {
        code: InfErr::MissingAccErr,
        cause: Some(format!("missing account {pk}")),
    }
}

pub(crate) fn missing_spl_data_err(mint: &[u8; 32]) -> InfError {
    let mint = Bs58PkString::encode(mint);
    InfError {
        code: InfErr::MissingSplDataErr,
        cause: Some(format!("missing spl pool account data for mint {mint}")),
    }
}

pub(crate) fn missing_svc_data_err(mint: &[u8; 32]) -> InfError {
    let mint = Bs58PkString::encode(mint);
    InfError {
        code: InfErr::MissingSvcDataErr,
        cause: Some(format!("missing sol value calculator data for mint {mint}")),
    }
}

pub(crate) fn unknown_pp_err(program_id: &[u8; 32]) -> InfError {
    let program_id = Bs58PkString::encode(program_id);
    InfError {
        code: InfErr::UnknownPpErr,
        cause: Some(format!("unknown pricing program {program_id}")),
    }
}

pub(crate) fn unknown_svc_err(program_id: &[u8; 32]) -> InfError {
    let program_id = Bs58PkString::encode(program_id);
    InfError {
        code: InfErr::UnknownSvcErr,
        cause: Some(format!("unknown sol value calculator program {program_id}")),
    }
}

pub(crate) fn no_valid_pda_err() -> InfError {
    InfError {
        code: InfErr::NoValidPdaErr,
        cause: None,
    }
}

pub(crate) fn unsupported_mint_err(mint: &[u8; 32]) -> InfError {
    let mint = Bs58PkString::encode(mint);
    InfError {
        code: InfErr::UnsupportedMintErr,
        cause: Some(format!("unsupported mint {mint}")),
    }
}

fn overflow_err() -> InfError {
    InfError {
        code: InfErr::InternalErr,
        cause: Some("overflow".to_owned()),
    }
}

fn zero_value_err() -> InfError {
    InfError {
        code: InfErr::UserErr,
        cause: Some("trade results in zero value, likely size too small".to_owned()),
    }
}

impl From<SplCalcErr> for InfError {
    #[inline]
    fn from(e: SplCalcErr) -> Self {
        const SPL_ERR_PREFIX: &str = "SplCalcErr::";

        let (code, cause) = match e {
            SplCalcErr::NotUpdated => (InfErr::PoolErr, format!("{SPL_ERR_PREFIX}{e}")),
            SplCalcErr::Ratio => (InfErr::InternalErr, format!("{SPL_ERR_PREFIX}{e}")),
        };

        InfError {
            code,
            cause: Some(cause),
        }
    }
}

impl From<LidoCalcErr> for InfError {
    #[inline]
    fn from(e: LidoCalcErr) -> Self {
        const LIDO_ERR_PREFIX: &str = "LidoCalcErr::";

        let (code, cause) = match e {
            LidoCalcErr::NotUpdated => (InfErr::PoolErr, format!("{LIDO_ERR_PREFIX}{e}")),
            LidoCalcErr::Ratio => (InfErr::InternalErr, format!("{LIDO_ERR_PREFIX}{e}")),
        };

        InfError {
            code,
            cause: Some(cause),
        }
    }
}

impl From<MarinadeCalcErr> for InfError {
    #[inline]
    fn from(e: MarinadeCalcErr) -> Self {
        const MARINADE_ERR_PREFIX: &str = "MarinadeCalcErr::";

        let (code, cause) = match e {
            MarinadeCalcErr::Paused | MarinadeCalcErr::StakeWithdrawDisabled => {
                (InfErr::PoolErr, format!("{MARINADE_ERR_PREFIX}{e}"))
            }
            MarinadeCalcErr::Ratio => (InfErr::InternalErr, format!("{MARINADE_ERR_PREFIX}{e}")),
        };
        InfError {
            code,
            cause: Some(cause),
        }
    }
}

impl<
        E1: Into<InfError>,
        E2: Into<InfError>,
        E3: Into<InfError>,
        E4: Into<InfError>,
        E5: Into<InfError>,
        E6: Into<InfError>,
    > From<SvcAg<E1, E2, E3, E4, E5, E6>> for InfError
{
    #[inline]
    fn from(e: SvcAg<E1, E2, E3, E4, E5, E6>) -> Self {
        match e {
            SvcAg::Lido(e) => e.into(),
            SvcAg::Marinade(e) => e.into(),
            SvcAg::SanctumSpl(e) => e.into(),
            SvcAg::SanctumSplMulti(e) => e.into(),
            SvcAg::Spl(e) => e.into(),
            SvcAg::Wsol(e) => e.into(),
        }
    }
}

impl From<FlatFeePricingErr> for InfError {
    #[inline]
    fn from(e: FlatFeePricingErr) -> Self {
        const FLAT_FEE_PRICING_ERR_PREFIX: &str = "FlatFeePricingErr::";

        let (code, cause) = match e {
            FlatFeePricingErr::Ratio => (
                InfErr::InternalErr,
                format!("{FLAT_FEE_PRICING_ERR_PREFIX}{e}"),
            ),
        };
        InfError {
            code,
            cause: Some(cause),
        }
    }
}

impl From<NotEnoughLiquidityErr> for InfError {
    fn from(e: NotEnoughLiquidityErr) -> Self {
        InfError {
            code: InfErr::PoolErr,
            cause: Some(e.to_string()),
        }
    }
}

#[allow(deprecated)]
impl<E1: Into<InfError>, E2: Into<InfError>> From<AddLiqQuoteErr<E1, E2>> for InfError {
    fn from(e: AddLiqQuoteErr<E1, E2>) -> Self {
        match e {
            AddLiqQuoteErr::InpCalc(e) => e.into(),
            AddLiqQuoteErr::Overflow => overflow_err(),
            AddLiqQuoteErr::ZeroValue => zero_value_err(),
            AddLiqQuoteErr::Pricing(e) => e.into(),
        }
    }
}

#[allow(deprecated)]
impl<E1: Into<InfError>, E2: Into<InfError>> From<RemoveLiqQuoteErr<E1, E2>> for InfError {
    fn from(e: RemoveLiqQuoteErr<E1, E2>) -> Self {
        match e {
            RemoveLiqQuoteErr::NotEnougLiquidity(e) => e.into(),
            RemoveLiqQuoteErr::OutCalc(e) => e.into(),
            RemoveLiqQuoteErr::Overflow => overflow_err(),
            RemoveLiqQuoteErr::Pricing(e) => e.into(),
            RemoveLiqQuoteErr::ZeroValue => zero_value_err(),
        }
    }
}

impl<E1: Into<InfError>, E2: Into<InfError>, E3: Into<InfError>> From<SwapQuoteErr<E1, E2, E3>>
    for InfError
{
    fn from(e: SwapQuoteErr<E1, E2, E3>) -> Self {
        match e {
            SwapQuoteErr::InpCalc(e) => e.into(),
            SwapQuoteErr::OutCalc(e) => e.into(),
            SwapQuoteErr::Overflow => overflow_err(),
            SwapQuoteErr::NotEnoughLiquidity(e) => e.into(),
            SwapQuoteErr::Pricing(e) => e.into(),
            SwapQuoteErr::ZeroValue => zero_value_err(),
        }
    }
}

impl From<FlatFeePricingColErr> for InfError {
    fn from(value: FlatFeePricingColErr) -> Self {
        match value {
            FlatFeePricingColErr::FeeAccountMissing { mint } => InfError {
                code: InfErr::MissingAccErr,
                cause: Some(format!(
                    "fee account for mint {} missing",
                    Bs58PkString::encode(&mint)
                )),
            },
            FlatFeePricingColErr::ProgramStateMissing => InfError {
                code: InfErr::MissingAccErr,
                cause: Some("flat fee program state missing".to_owned()),
            },
        }
    }
}

impl From<Infallible> for InfError {
    #[inline]
    fn from(_value: Infallible) -> Self {
        unreachable!()
    }
}

impl From<FlatFeePricingUpdateErr> for InfError {
    #[inline]
    fn from(value: FlatFeePricingUpdateErr) -> Self {
        match value {
            FlatFeePricingUpdateErr::AccDeser { pk } => acc_deser_err(&pk),
        }
    }
}

impl<E: Into<InfError>> From<PricingAg<E>> for InfError {
    #[inline]
    fn from(value: PricingAg<E>) -> Self {
        match value {
            PricingAg::FlatFee(e) => e.into(),
        }
    }
}

impl<E: Into<InfError>> From<UpdateErr<E>> for InfError {
    #[inline]
    fn from(value: UpdateErr<E>) -> Self {
        match value {
            UpdateErr::AccMissing { pk } => missing_acc_err(&pk),
            UpdateErr::Inner(e) => e.into(),
        }
    }
}
