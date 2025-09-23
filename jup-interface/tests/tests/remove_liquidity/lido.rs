use inf1_jup_interface::consts::INF_MINT_ADDR;
use inf1_std::inf1_svc_ag_std::inf1_svc_lido_core::solido_legacy_core::STSOL_MINT_ADDR;
use inf1_test_utils::{KeyedUiAccount, ALL_FIXTURES};
use jupiter_amm_interface::{QuoteParams, SwapMode};

use crate::common::{swap_test, SwapUserAccs};

fn fixtures_accs() -> SwapUserAccs<&'static str> {
    SwapUserAccs::default()
        .with_signer("inf-token-acc-owner")
        .with_inp_token_acc("inf-token-acc")
        .with_out_token_acc("stsol-token-acc")
}

#[test]
fn remove_liq_stsol_fixture_basic() {
    swap_test(
        QuoteParams {
            amount: 6969,
            input_mint: INF_MINT_ADDR.into(),
            output_mint: STSOL_MINT_ADDR.into(),
            swap_mode: SwapMode::ExactIn,
        },
        &ALL_FIXTURES,
        fixtures_accs().map(|n| KeyedUiAccount::from_test_fixtures_json(n).into_keyed_account()),
    );
}
