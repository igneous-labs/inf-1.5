use inf1_jup_interface::consts::WSOL_MINT_ADDR;
use inf1_std::inf1_svc_ag_std::{
    inf1_svc_lido_core::solido_legacy_core::STSOL_MINT_ADDR,
    inf1_svc_marinade_core::sanctum_marinade_liquid_staking_core::MSOL_MINT_ADDR,
};
use inf1_test_utils::{KeyedUiAccount, ALL_FIXTURES, JUPSOL_MINT};
use jupiter_amm_interface::{QuoteParams, SwapMode};
use solana_pubkey::Pubkey;

use crate::common::{swap_test, SwapUserAccs};

const QUOTE_PARAMS: QuoteParams = QuoteParams {
    amount: 1_000_000_000,
    input_mint: JUPSOL_MINT,
    output_mint: Pubkey::new_from_array([0u8; 32]),
    swap_mode: SwapMode::ExactIn,
};

fn fixtures_accs_base() -> SwapUserAccs<&'static str> {
    SwapUserAccs::default()
        .with_signer("jupsol-token-acc-owner")
        .with_inp_token_acc("jupsol-token-acc")
}

#[test]
fn swap_exact_in_jupsol_to_wsol_fixture_basic() {
    swap_test(
        QuoteParams {
            output_mint: WSOL_MINT_ADDR.into(),
            ..QUOTE_PARAMS
        },
        &ALL_FIXTURES,
        fixtures_accs_base()
            .with_out_token_acc("wsol-token-acc")
            .map(|n| KeyedUiAccount::from_test_fixtures_json(n).into_keyed_account()),
    );
}

#[test]
fn swap_exact_in_jupsol_to_msol_fixture_basic() {
    swap_test(
        QuoteParams {
            output_mint: MSOL_MINT_ADDR.into(),
            amount: 7698,
            ..QUOTE_PARAMS
        },
        &ALL_FIXTURES,
        fixtures_accs_base()
            .with_out_token_acc("msol-token-acc")
            .map(|n| KeyedUiAccount::from_test_fixtures_json(n).into_keyed_account()),
    );
}

#[test]
fn swap_exact_in_jupsol_to_stsol_fixture_basic() {
    swap_test(
        QuoteParams {
            output_mint: STSOL_MINT_ADDR.into(),
            amount: 6969,
            ..QUOTE_PARAMS
        },
        &ALL_FIXTURES,
        fixtures_accs_base()
            .with_out_token_acc("stsol-token-acc")
            .map(|n| KeyedUiAccount::from_test_fixtures_json(n).into_keyed_account()),
    );
}
