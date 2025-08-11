use inf1_jup_interface::consts::{INF_MINT_ADDR, WSOL_MINT_ADDR};
use jupiter_amm_interface::{QuoteParams, SwapMode};

use crate::common::{swap_test, KeyedUiAccount, SwapUserAccs, ALL_FIXTURES};

#[test]
fn remove_liq_wsol_fixture_basic() {
    swap_test(
        QuoteParams {
            amount: 1_000_000_000,
            input_mint: INF_MINT_ADDR.into(),
            output_mint: WSOL_MINT_ADDR.into(),
            swap_mode: SwapMode::ExactIn,
        },
        &ALL_FIXTURES,
        SwapUserAccs(
            ["inf-token-acc-owner", "inf-token-acc", "wsol-token-acc"]
                .map(|n| KeyedUiAccount::from_test_fixtures_json(n).into_keyed_account()),
        ),
    );
}
