use bs58_fixed_wasm::Bs58Array;
use inf1_svc_generic::{
    instructions::manager::{
        NewULUSIxAccsBuilder, ULUSIxData, ULUS_IX_IS_SIGNER, ULUS_IX_IS_WRITER,
    },
    keys::GLOBAL_CONST_KEYS_OWNED,
    pda::STATE_SEED,
};
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use tsify_next::Tsify;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    err::{no_valid_pda_err, InfError},
    instruction::{keys_signer_writable_to_metas, Instruction},
    interface::B58PK,
    pda::find_pda,
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct UpdateLastUpgradeSlotArgs {
    /// Current manager of the generic SOL value calculator.
    pub manager: B58PK,

    /// Generic SOL value calculator program ID.
    pub svc_program: B58PK,

    /// Pool program whose Loader-v3 upgrade slot is tracked by the calculator.
    pub pool_program: B58PK,
}

/// @throws
#[wasm_bindgen(js_name = updateLastUpgradeSlotIx)]
pub fn update_last_upgrade_slot_ix(
    UpdateLastUpgradeSlotArgs {
        manager: Bs58Array(manager),
        svc_program: Bs58Array(svc_program),
        pool_program: Bs58Array(pool_program),
    }: &UpdateLastUpgradeSlotArgs,
) -> Result<Instruction, InfError> {
    let state = find_pda(&[&STATE_SEED], svc_program)
        .ok_or_else(no_valid_pda_err)?
        .0;
    let pool_progdata = find_pda(&[pool_program], GLOBAL_CONST_KEYS_OWNED.bpf_loader_v3())
        .ok_or_else(no_valid_pda_err)?
        .0;

    let keys = NewULUSIxAccsBuilder::start()
        .with_manager(*manager)
        .with_state(state)
        .with_pool_prog(*pool_program)
        .with_pool_progdata(pool_progdata)
        .build();

    Ok(Instruction {
        data: ByteBuf::from(ULUSIxData::as_buf()),
        accounts: keys_signer_writable_to_metas(
            keys.0.iter(),
            ULUS_IX_IS_SIGNER.0.iter(),
            ULUS_IX_IS_WRITER.0.iter(),
        ),
        program_address: B58PK::new(*svc_program),
    })
}
