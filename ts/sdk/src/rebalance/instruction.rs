use bs58_fixed_wasm::Bs58Array;
use inf1_std::{
    inf1_ctl_core::{
        self,
        instructions::rebalance::{end::EndRebalanceIxData, start::StartRebalanceIxData},
    },
    inf1_pp_core::pair::Pair,
    rebalance::instruction::RebalanceIxArgs,
};
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use tsify_next::Tsify;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    err::InfError,
    instruction::{keys_signer_writable_to_metas, Instruction},
    interface::{PkPair, B58PK},
    Inf,
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi, large_number_types_as_bigints)]
#[serde(rename_all = "camelCase")]
pub struct RebalanceArgs {
    pub out: u64,
    pub min_starting_out_lst: u64,
    pub max_starting_inp_lst: u64,
    pub mints: PkPair,
    pub withdraw_to: B58PK,
}

#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi, large_number_types_as_bigints)]
#[serde(rename_all = "camelCase")]
pub struct RebalanceIxs {
    pub start: Instruction,
    pub end: Instruction,
}

/// @throws
#[wasm_bindgen(js_name = rebalanceIxs)]
pub fn rebalance_ixs(
    inf: &mut Inf,
    RebalanceArgs {
        mints:
            PkPair {
                inp: Bs58Array(inp_mint),
                out: Bs58Array(out_mint),
            },
        out,
        min_starting_out_lst,
        max_starting_inp_lst,
        withdraw_to: Bs58Array(withdraw_to),
    }: &RebalanceArgs,
) -> Result<RebalanceIxs, InfError> {
    let (start, end) = inf.0.rebalance_ixs_mut(&RebalanceIxArgs {
        out_amt: *out,
        min_starting_out_lst: *min_starting_out_lst,
        max_starting_inp_lst: *max_starting_inp_lst,
        mints: &Pair {
            inp: inp_mint,
            out: out_mint,
        },
        withdraw_to,
    })?;
    Ok(RebalanceIxs {
        start: Instruction {
            data: ByteBuf::from(StartRebalanceIxData::new(start.to_full()).as_buf()),
            accounts: keys_signer_writable_to_metas(
                start.accs.keys_owned().seq(),
                start.accs.is_signer().seq(),
                start.accs.is_writer().seq(),
            ),
            program_address: B58PK::new(inf1_ctl_core::ID),
        },
        end: Instruction {
            data: ByteBuf::from(EndRebalanceIxData::new().as_buf()),
            accounts: keys_signer_writable_to_metas(
                end.keys_owned().seq(),
                end.is_signer().seq(),
                end.is_writer().seq(),
            ),
            program_address: B58PK::new(inf1_ctl_core::ID),
        },
    })
}
