// In general, if an err conversion fn looks like it should belong in `.ok_or_else`, dont inline it.
// If it looks like it should belong in `.ok_or`, #[inline] is fine but likely wont make a difference.

use std::convert::Infallible;

use bs58_fixed::Bs58String;
use inf1_std::{
    err::{InfErr as InfStdErr, NotEnoughLiquidityErr},
    inf1_pp_ag_std::{
        inf1_pp_flatfee_std::{
            pricing::err::FlatFeePricingErr, traits::FlatFeePricingColErr,
            update::FlatFeePricingUpdateErr,
        },
        inf1_pp_flatslab_std::{
            pricing::FlatSlabPricingErr, traits::FlatSlabPricingColErr, typedefs::MintNotFoundErr,
            update::FlatSlabPricingUpdateErr,
        },
        PricingAg,
    },
    inf1_svc_ag_std::{
        inf1_svc_lido_core::calc::LidoCalcErr,
        inf1_svc_marinade_core::calc::MarinadeCalcErr,
        inf1_svc_spl_core::calc::SplCalcErr,
        update::{LidoUpdateErr, MarinadeUpdateErr, SplUpdateErr},
        SvcAg,
    },
    quote::{rebalance::RebalanceQuoteErr, swap::err::SwapQuoteErr},
    update::UpdateErr,
};
use serde::{Deserialize, Serialize};
use tsify_next::Tsify;
use wasm_bindgen::prelude::*;

#[allow(deprecated)]
use inf1_std::quote::liquidity::{add::AddLiqQuoteErr, remove::RemoveLiqQuoteErr};

type Bs58PkString = Bs58String<44>;

const ERR_CODE_MSG_SEP: &str = ":";

#[wasm_bindgen(typescript_custom_section)]
const ERROR_DECL: &'static str = r#"
export type ERR_CODE_MSG_SEP = ":";

/**
 * All {@link Error} objects thrown by the SDK have messages of this format
 */
export type InfErrMsg = `${InfErr}${ERR_CODE_MSG_SEP}${string}`;
"#;

/// Example-usage:
///
/// ```ignore
/// inf_err!(AccDeserErr, InternalErr);
/// ```
///
/// Generates
///
/// ```ignore
/// pub enum InfErr {
///     AccDeserErr,
///     InternalErr,
/// }
///
/// pub struct AllInfErrs (
///     pub [InfErr; 2],
/// );
///
/// pub fn all_inf_errs() -> AllInfErrs {
///     use InfErr::*;
///     AllInfErrs([
///         AccDeserErr,
///         InternalErr,
///     ])
/// }
///
/// ```
macro_rules! inf_err {
    (
        @ctr $ctr:expr;
        @seq { $($seq:tt)* };
        $variant:ident
        $(, $($tail:tt)*)?
    ) => {
        inf_err!(
            @ctr ($ctr + 1);
            @seq {
                $($seq)*
                $variant,
            };
            $($($tail)*)?
        );
    };

    // base-cases
    (
        @ctr $ctr:expr;
        @seq { $($seq:tt)* };
    ) => {
        /// All {@link Error} objects thrown by SDK functions will start with
        /// `{InfErr}:`, so that the `InfErr` error code can be
        /// extracted by splitting on the first colon `:`
        #[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Tsify)]
        #[tsify(into_wasm_abi, from_wasm_abi)]
        #[allow(clippy::enum_variant_names)] // we want all the ts consts to have `Err` suffix
        pub enum InfErr {
            $($seq)*
        }


        // TODO: ideally the ts type for this value is `[AccDeserErr, InternalErr, ...]`
        // instead of `[InfErr, InfErr]`, but the `type = ` tsify annotation is
        // horribly underpowered right now and can only take string literals, which we cannot
        // build up programmatically
        #[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Tsify)]
        #[tsify(into_wasm_abi, from_wasm_abi)]
        #[repr(transparent)]
        pub struct AllInfErrs(
            pub [InfErr; $ctr],
        );

        /// Returns the array of all possible {@link InfErr}s
        #[wasm_bindgen(js_name = allInfErrs)]
        pub fn all_inf_errs() -> AllInfErrs {
            use InfErr::*;
            AllInfErrs([
                $($seq)*
            ])
        }
    };
    () => {};

    // start
    ($($tail:tt)*) => { inf_err!(@ctr 0; @seq {}; $($tail)*); };
}

inf_err!(
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
    SizeTooSmallErr,
    SizeTooLargeErr,
);

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
        code: InfErr::SizeTooSmallErr,
        cause: Some("trade results in zero value".to_owned()),
    }
}

macro_rules! impl_from_acc_deser_err {
    ($Enum:ty) => {
        impl From<$Enum> for InfError {
            fn from(value: $Enum) -> Self {
                // `use` here because cannot do `$ty::AccDeser { ...`
                use $Enum::*;
                match value {
                    AccDeser { pk } => acc_deser_err(&pk),
                }
            }
        }
    };
}

impl From<InfStdErr> for InfError {
    fn from(value: InfStdErr) -> Self {
        match value {
            InfStdErr::AccDeser { pk } => acc_deser_err(&pk),
            InfStdErr::AddLiqQuote(e) => e.into(),
            InfStdErr::MissingAcc { pk } => missing_acc_err(&pk),
            InfStdErr::MissingSplData { mint } => missing_spl_data_err(&mint),
            InfStdErr::MissingSvcData { mint } => missing_svc_data_err(&mint),
            InfStdErr::NoValidPda => no_valid_pda_err(),
            InfStdErr::PricingProg(e) => e.into(),
            InfStdErr::RebalanceQuote(e) => e.into(),
            InfStdErr::RemoveLiqQuote(e) => e.into(),
            InfStdErr::SwapQuote(e) => e.into(),
            InfStdErr::UnknownPp { pp_prog_id } => unknown_pp_err(&pp_prog_id),
            InfStdErr::UnknownSvc { svc_prog_id } => unknown_svc_err(&svc_prog_id),
            InfStdErr::UnsupportedMint { mint } => unsupported_mint_err(&mint),
            InfStdErr::UpdatePp(e) => e.into(),
            InfStdErr::UpdateSvc(e) => e.into(),
        }
    }
}

// sol-val-calc programs

impl From<SplCalcErr> for InfError {
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

impl_from_acc_deser_err!(LidoUpdateErr);
impl_from_acc_deser_err!(MarinadeUpdateErr);
impl_from_acc_deser_err!(SplUpdateErr);

// Pricing programs

impl From<FlatFeePricingErr> for InfError {
    fn from(e: FlatFeePricingErr) -> Self {
        const ERR_PREFIX: &str = "FlatFeePricingErr::";

        let (code, cause) = match e {
            FlatFeePricingErr::Ratio => (InfErr::InternalErr, format!("{ERR_PREFIX}{e}")),
        };
        InfError {
            code,
            cause: Some(cause),
        }
    }
}

impl From<FlatSlabPricingErr> for InfError {
    fn from(e: FlatSlabPricingErr) -> Self {
        const ERR_PREFIX: &str = "FlatSlabPricingErr::";

        let (code, cause) = match e {
            FlatSlabPricingErr::Ratio | FlatSlabPricingErr::NetNegativeFees => {
                (InfErr::InternalErr, format!("{ERR_PREFIX}{e}"))
            }
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
            code: InfErr::SizeTooLargeErr,
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

impl<E1: Into<InfError>, E2: Into<InfError>> From<RebalanceQuoteErr<E1, E2>> for InfError {
    fn from(e: RebalanceQuoteErr<E1, E2>) -> Self {
        match e {
            RebalanceQuoteErr::InpCalc(e) => e.into(),
            RebalanceQuoteErr::OutCalc(e) => e.into(),
            RebalanceQuoteErr::NotEnoughLiquidity(e) => e.into(),
            RebalanceQuoteErr::Overflow => overflow_err(),
        }
    }
}

#[allow(deprecated)]
impl<E1: Into<InfError>, E2: Into<InfError>> From<RemoveLiqQuoteErr<E1, E2>> for InfError {
    fn from(e: RemoveLiqQuoteErr<E1, E2>) -> Self {
        match e {
            RemoveLiqQuoteErr::NotEnoughLiquidity(e) => e.into(),
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

impl From<FlatSlabPricingColErr> for InfError {
    fn from(value: FlatSlabPricingColErr) -> Self {
        match value {
            FlatSlabPricingColErr::MintNotFound(MintNotFoundErr { mint, .. }) => InfError {
                code: InfErr::MissingAccErr,
                cause: Some(format!(
                    "slab entry for mint {} missing",
                    Bs58PkString::encode(&mint)
                )),
            },
        }
    }
}

impl From<Infallible> for InfError {
    fn from(_value: Infallible) -> Self {
        unreachable!()
    }
}

impl_from_acc_deser_err!(FlatFeePricingUpdateErr);
impl_from_acc_deser_err!(FlatSlabPricingUpdateErr);

impl<FlatFee: Into<InfError>, FlatSlab: Into<InfError>> From<PricingAg<FlatFee, FlatSlab>>
    for InfError
{
    fn from(value: PricingAg<FlatFee, FlatSlab>) -> Self {
        match value {
            PricingAg::FlatFee(e) => e.into(),
            PricingAg::FlatSlab(e) => e.into(),
        }
    }
}

impl<E: Into<InfError>> From<UpdateErr<E>> for InfError {
    fn from(value: UpdateErr<E>) -> Self {
        match value {
            UpdateErr::AccMissing { pk } => missing_acc_err(&pk),
            UpdateErr::Inner(e) => e.into(),
        }
    }
}
