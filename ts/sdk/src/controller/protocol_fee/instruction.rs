use bs58_fixed_wasm::Bs58Array;
use inf1_std::inf1_ctl_core::{
    self,
    instructions::protocol_fee::withdraw_protocol_fees::v2::{
        NewWithdrawProtocolFeesV2IxAccsBuilder, WithdrawProtocolFeesV2IxData,
        WITHDRAW_PROTOCOL_FEES_V2_IX_IS_SIGNER, WITHDRAW_PROTOCOL_FEES_V2_IX_IS_WRITER,
    },
    keys::POOL_STATE_ID,
};
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use tsify_next::Tsify;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    err::InfError,
    instruction::{keys_signer_writable_to_metas, Instruction},
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
#[wasm_bindgen(js_name = withdrawProtocolFeesV2IxRaw)]
pub fn withdraw_protocol_fees_v2_ix_raw(
    WithdrawProtocolFeesV2Args {
        protocol_fee_beneficiary: Bs58Array(protocol_fee_beneficiary),
        withdraw_to: Bs58Array(withdraw_to),
        inf_mint: Bs58Array(inf_mint),
        token_program: Bs58Array(token_program),
    }: &WithdrawProtocolFeesV2Args,
) -> Result<Instruction, InfError> {
    println!("withdraw_protocol_fees_v2_ix_raw: blah");
    let keys = NewWithdrawProtocolFeesV2IxAccsBuilder::start()
        .with_pool_state(POOL_STATE_ID)
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
        program_address: B58PK::new(inf1_ctl_core::ID),
    })
}
