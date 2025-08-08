use inf1_std::InfStd;
use wasm_bindgen::prelude::*;

mod err;
mod init;
mod instruction;
mod interface;
mod pda;
mod spl;
mod trade;

#[derive(Debug, Clone, PartialEq, Eq)]
#[wasm_bindgen]
#[repr(transparent)]
pub struct Inf(pub(crate) InfStd);
