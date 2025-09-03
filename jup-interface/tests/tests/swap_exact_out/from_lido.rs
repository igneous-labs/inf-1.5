use inf1_jup_interface::consts::WSOL_MINT_ADDR;
use inf1_std::inf1_svc_ag_std::{
    inf1_svc_lido_core::solido_legacy_core::STSOL_MINT_ADDR,
    inf1_svc_marinade_core::sanctum_marinade_liquid_staking_core::MSOL_MINT_ADDR,
};
use inf1_test_utils::{KeyedUiAccount, ALL_FIXTURES};
use jupiter_amm_interface::{QuoteParams, SwapMode};
use solana_pubkey::Pubkey;

use crate::common::{swap_test, SwapUserAccs, JUPSOL_MINT_ADDR};

const QUOTE_PARAMS: QuoteParams = QuoteParams {
    amount: 1_000_000_000,
    input_mint: Pubkey::new_from_array(STSOL_MINT_ADDR),
    output_mint: Pubkey::new_from_array([0u8; 32]),
    swap_mode: SwapMode::ExactOut,
};

fn fixtures_accs_base() -> SwapUserAccs<&'static str> {
    SwapUserAccs::default()
        .with_signer("stsol-token-acc-owner")
        .with_inp_token_acc("stsol-token-acc")
}

#[test]
fn swap_exact_out_stsol_to_wsol_fixture_basic() {
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
fn swap_exact_out_stsol_to_jupsol_fixture_basic() {
    swap_test(
        QuoteParams {
            output_mint: JUPSOL_MINT_ADDR.into(),
            ..QUOTE_PARAMS
        },
        &ALL_FIXTURES,
        fixtures_accs_base()
            .with_out_token_acc("jupsol-token-acc")
            .map(|n| KeyedUiAccount::from_test_fixtures_json(n).into_keyed_account()),
    );
}

#[test]
fn swap_exact_out_stsol_to_msol_fixture_basic() {
    swap_test(
        QuoteParams {
            output_mint: MSOL_MINT_ADDR.into(),
            amount: 6969,
            ..QUOTE_PARAMS
        },
        &ALL_FIXTURES,
        fixtures_accs_base()
            .with_out_token_acc("msol-token-acc")
            .map(|n| KeyedUiAccount::from_test_fixtures_json(n).into_keyed_account()),
    );
}
