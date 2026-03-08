//! Types to bridge the wasm serde interface

use std::collections::HashMap;

use bs58_fixed_wasm::Bs58Array;
use inf1_std::update::UpdateMap;
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use tsify_next::{declare, Tsify};

#[declare]
pub type B58PK = Bs58Array<32, 44>;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi, large_number_types_as_bigints)]
#[serde(rename_all = "camelCase")]
pub struct PkPair {
    pub inp: B58PK,
    pub out: B58PK,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct AccountMap(pub HashMap<B58PK, Account>);

impl inf1_std::update::Account for Account {
    #[inline]
    fn data(&self) -> &[u8] {
        &self.data
    }
}

impl UpdateMap for AccountMap {
    type Account<'a> = &'a Account;

    #[inline]
    fn get_account(&self, pk: &[u8; 32]) -> Option<Self::Account<'_>> {
        self.0.get(&Bs58Array(*pk))
    }
}

/// Map of `mint: stake pool account` for spl (all deploys) LSTs.
///
/// This data is required to determine how to properly initialize the corresponding
/// sol value calculator data since which stake pool account corresponds to which mint
/// is not available onchain (yet)
#[derive(Debug, Default, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct SplPoolAccounts(pub HashMap<B58PK, B58PK>);

#[derive(Debug, Default, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi, large_number_types_as_bigints)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub data: ByteBuf,
    pub owner: B58PK,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi, large_number_types_as_bigints)]
#[serde(rename_all = "camelCase")]
pub struct PoolStateV2 {
    pub total_sol_value: u64,
    pub protocol_fee_nanos: u32,
    pub version: u8,
    pub is_disabled: u8,
    pub is_rebalancing: u8,
    pub admin: B58PK,
    pub rebalance_authority: B58PK,
    pub protocol_fee_beneficiary: B58PK,
    pub pricing_program: B58PK,
    pub lp_token_mint: B58PK,
    pub rps_authority: B58PK,
    pub rps: u64,
    pub withheld_lamports: u64,
    pub protocol_fee_lamports: u64,
    pub last_release_slot: u64,
}

pub const fn pool_state_v2_from_intf(
    PoolStateV2 {
        total_sol_value,
        protocol_fee_nanos,
        version,
        is_disabled,
        is_rebalancing,
        admin: Bs58Array(admin),
        rebalance_authority: Bs58Array(rebalance_authority),
        protocol_fee_beneficiary: Bs58Array(protocol_fee_beneficiary),
        pricing_program: Bs58Array(pricing_program),
        lp_token_mint: Bs58Array(lp_token_mint),
        rps_authority: Bs58Array(rps_authority),
        rps,
        withheld_lamports,
        protocol_fee_lamports,
        last_release_slot,
    }: PoolStateV2,
) -> inf1_std::inf1_ctl_core::accounts::pool_state::PoolStateV2 {
    inf1_std::inf1_ctl_core::accounts::pool_state::PoolStateV2 {
        total_sol_value,
        protocol_fee_nanos,
        version,
        is_disabled,
        is_rebalancing,
        padding: [0],
        rps,
        withheld_lamports,
        protocol_fee_lamports,
        last_release_slot,
        admin,
        rebalance_authority,
        protocol_fee_beneficiary,
        pricing_program,
        lp_token_mint,
        rps_authority,
    }
}

pub const fn pool_state_v2_into_intf(
    inf1_std::inf1_ctl_core::accounts::pool_state::PoolStateV2 {
        total_sol_value,
        protocol_fee_nanos,
        version,
        is_disabled,
        is_rebalancing,
        rps,
        withheld_lamports,
        protocol_fee_lamports,
        last_release_slot,
        admin,
        rebalance_authority,
        protocol_fee_beneficiary,
        pricing_program,
        lp_token_mint,
        rps_authority,
        padding: _,
    }: inf1_std::inf1_ctl_core::accounts::pool_state::PoolStateV2,
) -> PoolStateV2 {
    PoolStateV2 {
        total_sol_value,
        admin: B58PK::new(admin),
        is_disabled,
        is_rebalancing,
        lp_token_mint: B58PK::new(lp_token_mint),
        pricing_program: B58PK::new(pricing_program),
        protocol_fee_beneficiary: B58PK::new(protocol_fee_beneficiary),
        rebalance_authority: B58PK::new(rebalance_authority),
        version,
        protocol_fee_nanos,
        rps_authority: B58PK::new(rps_authority),
        rps,
        withheld_lamports,
        protocol_fee_lamports,
        last_release_slot,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi, large_number_types_as_bigints)]
#[serde(rename_all = "camelCase")]
pub struct LstState {
    pub is_input_disabled: u8,
    pub pool_reserves_bump: u8,
    pub protocol_fee_accumulator_bump: u8,
    pub sol_value: u64,
    pub mint: B58PK,
    pub sol_value_calculator: B58PK,
}

pub const fn lst_state_from_intf(
    LstState {
        is_input_disabled,
        mint: Bs58Array(mint),
        pool_reserves_bump,
        protocol_fee_accumulator_bump,
        sol_value,
        sol_value_calculator: Bs58Array(sol_value_calculator),
    }: LstState,
) -> inf1_std::inf1_ctl_core::typedefs::lst_state::LstState {
    inf1_std::inf1_ctl_core::typedefs::lst_state::LstState {
        is_input_disabled,
        mint,
        pool_reserves_bump,
        protocol_fee_accumulator_bump,
        sol_value,
        sol_value_calculator,
        padding: [0; 5],
    }
}

pub const fn lst_state_into_intf(
    inf1_std::inf1_ctl_core::typedefs::lst_state::LstState {
        is_input_disabled,
        mint,
        pool_reserves_bump,
        protocol_fee_accumulator_bump,
        sol_value,
        sol_value_calculator,
        padding: _,
    }: inf1_std::inf1_ctl_core::typedefs::lst_state::LstState,
) -> LstState {
    LstState {
        is_input_disabled,
        mint: B58PK::new(mint),
        pool_reserves_bump,
        protocol_fee_accumulator_bump,
        sol_value,
        sol_value_calculator: B58PK::new(sol_value_calculator),
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi, large_number_types_as_bigints)]
#[serde(rename_all = "camelCase")]
pub enum SlotLookahead {
    /// Lookahead to this absolute slot number
    Abs(u64),

    /// Lookahead relative, to `slot = pool.last_release_slot + this`
    Rel(u64),
}
