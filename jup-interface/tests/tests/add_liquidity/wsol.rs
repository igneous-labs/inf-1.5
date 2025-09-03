use inf1_jup_interface::consts::{INF_MINT_ADDR, WSOL_MINT_ADDR};
use inf1_test_utils::{KeyedUiAccount, ALL_FIXTURES};
use jupiter_amm_interface::{QuoteParams, SwapMode};

use crate::common::{swap_test, SwapUserAccs};

fn fixtures_accs() -> SwapUserAccs<&'static str> {
    SwapUserAccs::default()
        .with_signer("wsol-token-acc-owner")
        .with_inp_token_acc("wsol-token-acc")
        .with_out_token_acc("inf-token-acc")
}

#[test]
fn add_liq_wsol_fixture_basic() {
    swap_test(
        QuoteParams {
            amount: 1_000_000_000,
            input_mint: WSOL_MINT_ADDR.into(),
            output_mint: INF_MINT_ADDR.into(),
            swap_mode: SwapMode::ExactIn,
        },
        &ALL_FIXTURES,
        fixtures_accs().map(|n| KeyedUiAccount::from_test_fixtures_json(n).into_keyed_account()),
    );
}
