use inf1_std::{inf1_ctl_core::accounts::pool_state::PoolState, inf1_pp_ag_std::PricingAgTy};
use solana_pubkey::Pubkey;

pub const LABEL: &str = "Sanctum Infinity";

pub const INF_MINT_ADDR: [u8; 32] =
    Pubkey::from_str_const("5oVNBeEEQvYi1cX3ir8Dx5n1P7pdxydbGF2X4TxVusJm").to_bytes();

pub const WSOL_MINT_ADDR: [u8; 32] =
    Pubkey::from_str_const("So11111111111111111111111111111111111111112").to_bytes();

/// A dummy mainnet pool that tries to use the latest values of mainnet vars
/// for vars that affect [`jupiter_amm_interface::Amm::get_accounts_to_update`]
/// so that [`crate::Inf`] only needs 1 more update cycle before it's functioning
pub const DEFAULT_MAINNET_POOL: PoolState = PoolState {
    pricing_program: *PricingAgTy::FlatFee(()).program_id(),
    lp_token_mint: INF_MINT_ADDR,

    // dont-cares, since they will be
    // replaced in the first update cycle
    version: 0,
    is_disabled: 0,
    is_rebalancing: 0,
    total_sol_value: 0,
    trading_protocol_fee_bps: 0,
    lp_protocol_fee_bps: 0,

    // dont-cares, since they dont affect
    // jup functionality at all
    padding: [0],
    admin: [0; 32],
    rebalance_authority: [0; 32],
    protocol_fee_beneficiary: [0; 32],
};

/// TODO: figure out how to make this dynamic so that we can add spl stake pools
/// without updating the crate.
///
/// Array of `(spl_lst_mints, spl_stake_pool_addr)`
pub const SPL_LSTS: [([u8; 32], [u8; 32]); 1] = [([0u8; 32], [0u8; 32])];
