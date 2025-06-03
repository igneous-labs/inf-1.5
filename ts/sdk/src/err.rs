use std::fmt::Display;

use bs58_fixed::Bs58String;
use wasm_bindgen::{intern, JsError};

type Bs58PkString = Bs58String<44>;

pub(crate) fn no_valid_pda_err() -> JsError {
    JsError::new(intern("no valid PDA found"))
}

pub(crate) fn missing_acc_err(pk: &[u8; 32]) -> JsError {
    JsError::new(&format!("missing account {}", Bs58PkString::encode(pk)))
}

pub(crate) fn acc_deser_err(pk: &[u8; 32]) -> JsError {
    JsError::new(&format!(
        "account data for {} not of expected format",
        Bs58PkString::encode(pk)
    ))
}

pub(crate) fn unknown_svc_err(program_id: &[u8; 32]) -> JsError {
    JsError::new(&format!(
        "unknown sol value calculator program {}",
        Bs58PkString::encode(program_id)
    ))
}

pub(crate) fn missing_spl_data(mint: &[u8; 32]) -> JsError {
    JsError::new(&format!(
        "missing spl pool account data for mint {}",
        Bs58PkString::encode(mint)
    ))
}

pub(crate) fn generic_err(e: impl Display) -> JsError {
    JsError::new(&format!("{e}"))
}
