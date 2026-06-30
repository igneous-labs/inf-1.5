use bs58_fixed_wasm::Bs58Array;
use inf1_std::{
    inf1_ctl_core::token_info::TokenInfo,
    pda::{find_ata, CONST_PDAS, LST_STATE_LIST_SEED, POOL_STATE_SEED, PROTOCOL_FEE_SEED},
};
use serde::{Deserialize, Serialize};
use tsify_next::Tsify;
use wasm_bindgen::prelude::*;

use crate::{
    err::{no_valid_pda_err, InfError},
    interface::B58PK,
    pda::{find_pda, FoundPda},
};

#[derive(Debug, Default, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi, large_number_types_as_bigints)]
#[serde(rename_all = "camelCase")]
pub struct FindPoolAtaArgs {
    /// Controller program ID.
    /// Default = `5ocnV1qiCgaQR8Jb8xWnVbApfaygJ8tNoZfgPwsgx9kx`
    #[tsify(optional)]
    pub prog_id: Option<B58PK>,

    pub mint: B58PK,
}

/// @throws if no valid PDA found
/// TODO: token-22 support
#[wasm_bindgen(js_name = findPoolReservesAta)]
pub fn find_pool_reserves_ata(
    FindPoolAtaArgs {
        prog_id,
        mint: Bs58Array(mint),
    }: &FindPoolAtaArgs,
) -> Result<FoundPda, InfError> {
    let (auth, _) = find_pool_state(prog_id.as_ref().map(|Bs58Array(p)| p))?;
    find_ata(find_pda, &auth, &TokenInfo::tokenkeg(mint))
        .ok_or_else(no_valid_pda_err)
        .map(Into::into)
}

/// @deprecated Protocol fee accumulator token accounts are no longer used in v2
///
/// @throws if no valid PDA found
/// TODO: token-22 support
#[wasm_bindgen(js_name = findProtocolFeeAccumulatorAta)]
pub fn find_protocol_fee_accumulator_ata(
    FindPoolAtaArgs {
        prog_id,
        mint: Bs58Array(mint),
    }: &FindPoolAtaArgs,
) -> Result<FoundPda, InfError> {
    let (auth, _) = find_protocol_fee(prog_id.as_ref().map(|Bs58Array(p)| p))?;
    find_ata(find_pda, &auth, &TokenInfo::tokenkeg(mint))
        .ok_or_else(no_valid_pda_err)
        .map(Into::into)
}

pub(crate) fn find_pool_state(prog_id: Option<&[u8; 32]>) -> Result<([u8; 32], u8), InfError> {
    find_const_pda(prog_id, *CONST_PDAS.pool_state(), &[&POOL_STATE_SEED])
}

pub(crate) fn find_lst_state_list(prog_id: Option<&[u8; 32]>) -> Result<([u8; 32], u8), InfError> {
    find_const_pda(
        prog_id,
        *CONST_PDAS.lst_state_list(),
        &[&LST_STATE_LIST_SEED],
    )
}

pub(crate) fn find_protocol_fee(prog_id: Option<&[u8; 32]>) -> Result<([u8; 32], u8), InfError> {
    find_const_pda(prog_id, *CONST_PDAS.protocol_fee(), &[&PROTOCOL_FEE_SEED])
}

fn find_const_pda(
    prog_id: Option<&[u8; 32]>,
    default: ([u8; 32], u8),
    seeds: &[&[u8]],
) -> Result<([u8; 32], u8), InfError> {
    prog_id
        .map_or_else(|| Some(default), |p| find_pda(seeds, p))
        .ok_or_else(no_valid_pda_err)
}
