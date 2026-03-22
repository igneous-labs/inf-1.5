use bs58_fixed_wasm::Bs58Array;
use inf1_std::inf1_ctl_core::{
    self, instructions::protocol_fee::withdraw_protocol_fees::v2::WithdrawProtocolFeesV2IxData,
    pda::const_find_pool_state,
};
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use tsify_next::Tsify;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    err::InfError,
    instruction::{AccountMeta, Instruction, Role},
    interface::B58PK,
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct WithdrawProtocolFeesV2Args {
    pub protocol_fee_beneficiary: B58PK,
    pub withdraw_to: B58PK,
    pub inf_mint: B58PK,
    pub token_program: B58PK,
}

/// @throws
#[wasm_bindgen(js_name = withdrawProtocolFeesV2Ix)]
pub fn withdraw_protocol_fees_v2_ix(
    WithdrawProtocolFeesV2Args {
        protocol_fee_beneficiary: Bs58Array(protocol_fee_beneficiary),
        withdraw_to: Bs58Array(withdraw_to),
        inf_mint: Bs58Array(inf_mint),
        token_program: Bs58Array(token_program),
    }: &WithdrawProtocolFeesV2Args,
) -> Result<Instruction, InfError> {
    let pool_state = const_find_pool_state(&inf1_ctl_core::ID).0;

    Ok(Instruction {
        data: ByteBuf::from(WithdrawProtocolFeesV2IxData::as_buf()),
        accounts: [
            AccountMeta::new(pool_state, Role::Writable),
            AccountMeta::new(*protocol_fee_beneficiary, Role::ReadonlySigner),
            AccountMeta::new(*withdraw_to, Role::Writable),
            AccountMeta::new(*inf_mint, Role::Writable),
            AccountMeta::new(*token_program, Role::Readonly),
        ]
        .into(),
        program_address: B58PK::new(inf1_ctl_core::ID),
    })
}
