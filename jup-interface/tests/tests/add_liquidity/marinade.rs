use inf1_jup_interface::consts::INF_MINT_ADDR;
use inf1_std::inf1_svc_ag_std::inf1_svc_marinade_core::sanctum_marinade_liquid_staking_core::MSOL_MINT_ADDR;
use jupiter_amm_interface::{QuoteParams, SwapMode};

use crate::common::{swap_test, KeyedUiAccount, SwapUserAccs, ALL_FIXTURES};

#[test]
fn add_liq_msol_fixture_basic() {
    swap_test(
        QuoteParams {
            amount: 1_000_000_000,
            input_mint: MSOL_MINT_ADDR.into(),
            output_mint: INF_MINT_ADDR.into(),
            swap_mode: SwapMode::ExactIn,
        },
        &ALL_FIXTURES,
        SwapUserAccs(
            ["msol-token-acc-owner", "msol-token-acc", "inf-token-acc"]
                .map(|n| KeyedUiAccount::from_test_fixtures_json(n).into_keyed_account()),
        ),
    );
}
