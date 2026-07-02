use bs58_fixed_wasm::Bs58Array;
use inf1_std::inf1_ctl_core::{
    instructions::protocol_fee::withdraw_protocol_fees::v2::{
        NewWithdrawProtocolFeesV2IxAccsBuilder, WithdrawProtocolFeesV2IxData,
        WITHDRAW_PROTOCOL_FEES_V2_IX_IS_SIGNER, WITHDRAW_PROTOCOL_FEES_V2_IX_IS_WRITER,
    },
    keys::CONST_KEYS_OWNED,
};
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use tsify_next::Tsify;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    err::InfError,
    instruction::{keys_signer_writable_to_metas, Instruction},
    interface::B58PK,
    pda::controller::find_pool_state,
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct WithdrawProtocolFeesV2Args {
    pub protocol_fee_beneficiary: B58PK,
    pub withdraw_to: B58PK,
    pub inf_mint: B58PK,
    pub token_program: B58PK,

    /// Controller program ID.
    /// Default = `5ocnV1qiCgaQR8Jb8xWnVbApfaygJ8tNoZfgPwsgx9kx`
    #[tsify(optional)]
    pub prog_id: Option<B58PK>,
}

/// @throws
#[wasm_bindgen(js_name = withdrawProtocolFeesV2IxRaw)]
pub fn withdraw_protocol_fees_v2_ix_raw(
    WithdrawProtocolFeesV2Args {
        protocol_fee_beneficiary: Bs58Array(protocol_fee_beneficiary),
        withdraw_to: Bs58Array(withdraw_to),
        inf_mint: Bs58Array(inf_mint),
        token_program: Bs58Array(token_program),
        prog_id,
    }: &WithdrawProtocolFeesV2Args,
) -> Result<Instruction, InfError> {
    let keys = NewWithdrawProtocolFeesV2IxAccsBuilder::start()
        .with_pool_state(find_pool_state(prog_id.as_ref().map(|Bs58Array(p)| p))?.0)
        .with_beneficiary(*protocol_fee_beneficiary)
        .with_withdraw_to(*withdraw_to)
        .with_inf_mint(*inf_mint)
        .with_token_program(*token_program)
        .build();

    Ok(Instruction {
        data: ByteBuf::from(WithdrawProtocolFeesV2IxData::as_buf()),
        accounts: keys_signer_writable_to_metas(
            keys.0.iter(),
            WITHDRAW_PROTOCOL_FEES_V2_IX_IS_SIGNER.0.iter(),
            WITHDRAW_PROTOCOL_FEES_V2_IX_IS_WRITER.0.iter(),
        ),
        program_address: prog_id.unwrap_or_else(|| B58PK::new(*CONST_KEYS_OWNED.program())),
    })
}
