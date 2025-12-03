use inf1_ctl_jiminy::instructions::swap::v2::{
    exact_out::NewSwapExactOutV2IxPreAccsBuilder, IxPreAccs,
};
use inf1_test_utils::KeyedUiAccount;
use solana_account::Account;
use solana_pubkey::Pubkey;

mod exact_in;
mod exact_out;

fn jupsol_to_wsol_prefix_fixtures() -> IxPreAccs<(Pubkey, Account)> {
    IxPreAccs(
        NewSwapExactOutV2IxPreAccsBuilder::start()
            .with_signer("jupsol-token-acc-owner")
            .with_pool_state("pool-state")
            .with_lst_state_list("lst-state-list")
            .with_inp_acc("jupsol-token-acc")
            .with_inp_mint("jupsol-mint")
            .with_inp_pool_reserves("jupsol-reserves")
            .with_out_acc("wsol-token-acc")
            .with_out_mint("wsol-mint")
            .with_out_pool_reserves("wsol-reserves")
            // TODO: loading these 2 large program accounts might be slow
            .with_inp_token_program("tokenkeg")
            .with_out_token_program("tokenkeg")
            .build()
            .0
            .map(|n| KeyedUiAccount::from_test_fixtures_json(n).into_keyed_account()),
    )
}
