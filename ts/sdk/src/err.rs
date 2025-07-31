use std::convert::Infallible;

use bs58_fixed::Bs58String;
use inf1_core::{err::NotEnoughLiquidityErr, quote::swap::err::SwapQuoteErr};
use inf1_pp_flatfee_core::pricing::err::FlatFeePricingErr;
use inf1_svc_ag::{
    calc::CalcAgErr, inf1_svc_lido_core::calc::LidoCalcErr,
    inf1_svc_marinade_core::calc::MarinadeCalcErr, inf1_svc_spl_core::calc::SplCalcErr,
};
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

pub(crate) fn calc_ag_err(e: CalcAgErr) -> InfError {
    match e {
        CalcAgErr::Lido(e) => lido_calc_err(e),
        CalcAgErr::Marinade(e) => marinade_calc_err(e),
        CalcAgErr::Spl(e) => spl_calc_err(e),
    }
}

fn spl_calc_err(e: SplCalcErr) -> InfError {
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

fn lido_calc_err(e: LidoCalcErr) -> InfError {
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

fn marinade_calc_err(e: MarinadeCalcErr) -> InfError {
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

fn flat_fee_pricing_err(e: FlatFeePricingErr) -> InfError {
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

#[allow(deprecated)]
pub(crate) fn add_liq_quote_err(e: AddLiqQuoteErr<CalcAgErr, Infallible>) -> InfError {
    match e {
        AddLiqQuoteErr::InpCalc(e) => calc_ag_err(e),
        AddLiqQuoteErr::Overflow => overflow_err(),
        AddLiqQuoteErr::ZeroValue => zero_value_err(),
        // FlatFeeProgram does not do anything for PriceLpTokensToMint, so Infallible
        AddLiqQuoteErr::Pricing(_e) => unreachable!(),
    }
}

#[allow(deprecated)]
pub(crate) fn remove_liq_quote_err(e: RemoveLiqQuoteErr<CalcAgErr, FlatFeePricingErr>) -> InfError {
    match e {
        RemoveLiqQuoteErr::NotEnougLiquidity(e) => not_enough_liquidity_err(e),
        RemoveLiqQuoteErr::OutCalc(e) => calc_ag_err(e),
        RemoveLiqQuoteErr::Overflow => overflow_err(),
        RemoveLiqQuoteErr::Pricing(e) => flat_fee_pricing_err(e),
        RemoveLiqQuoteErr::ZeroValue => zero_value_err(),
    }
}

pub(crate) fn swap_quote_err(e: SwapQuoteErr<CalcAgErr, CalcAgErr, FlatFeePricingErr>) -> InfError {
    match e {
        SwapQuoteErr::InpCalc(e) | SwapQuoteErr::OutCalc(e) => calc_ag_err(e),
        SwapQuoteErr::Overflow => overflow_err(),
        SwapQuoteErr::NotEnoughLiquidity(e) => not_enough_liquidity_err(e),
        SwapQuoteErr::Pricing(e) => flat_fee_pricing_err(e),
        SwapQuoteErr::ZeroValue => zero_value_err(),
    }
}

fn overflow_err() -> InfError {
    InfError {
        code: InfErr::InternalErr,
        cause: Some("overflow".to_owned()),
    }
}

fn not_enough_liquidity_err(e: NotEnoughLiquidityErr) -> InfError {
    InfError {
        code: InfErr::PoolErr,
        cause: Some(e.to_string()),
    }
}

fn zero_value_err() -> InfError {
    InfError {
        code: InfErr::UserErr,
        cause: Some("trade results in zero value, likely size too small".to_owned()),
    }
}
