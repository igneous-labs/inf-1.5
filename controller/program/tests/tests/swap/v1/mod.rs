use inf1_ctl_jiminy::instructions::swap as ctl_swap;
use inf1_std::instructions::{liquidity, swap::IxAccs};
use inf1_test_utils::{fill_mock_prog_accs, AccountMap, KeyedUiAccount};
use solana_account::Account;
use solana_pubkey::Pubkey;

use crate::tests::swap::{V1Args, V2Args};

mod add_liq;
mod exact_in;
mod exact_out;
mod rem_liq;

fn args_to_v2<P>(
    V1Args {
        inp_lst_index,
        out_lst_index,
        limit,
        amount,
        accs:
            IxAccs {
                ix_prefix,
                inp_calc_prog,
                inp_calc,
                out_calc_prog,
                out_calc,
                pricing_prog,
                pricing,
            },
    }: V1Args<P>,
) -> V2Args<P> {
    V2Args {
        inp_lst_index,
        out_lst_index,
        limit,
        amount,
        accs: IxAccs {
            ix_prefix: ix_prefix.into(),
            inp_calc_prog,
            inp_calc,
            out_calc_prog,
            out_calc,
            pricing_prog,
            pricing,
        },
    }
}

fn fill_liq_prog_accs<I, C, P>(
    am: &mut AccountMap,
    liquidity::IxAccs {
        lst_calc_prog,
        pricing_prog,
        ..
    }: &liquidity::IxAccs<[u8; 32], I, C, P>,
) {
    fill_mock_prog_accs(am, [*lst_calc_prog, *pricing_prog]);
}

fn jupsol_to_msol_prefix_fixtures() -> ctl_swap::v1::IxPreAccs<(Pubkey, Account)> {
    ctl_swap::v1::IxPreAccs(
        ctl_swap::v1::NewIxPreAccsBuilder::start()
            .with_signer("jupsol-token-acc-owner")
            .with_pool_state("pool-state")
            .with_lst_state_list("lst-state-list")
            .with_inp_lst_acc("jupsol-token-acc")
            .with_inp_lst_mint("jupsol-mint")
            .with_inp_pool_reserves("jupsol-reserves")
            .with_out_lst_acc("msol-token-acc")
            .with_out_lst_mint("msol-mint")
            .with_out_pool_reserves("msol-reserves")
            .with_inp_lst_token_program("tokenkeg")
            .with_out_lst_token_program("tokenkeg")
            .with_protocol_fee_accumulator("msol-pf-accum")
            .build()
            .0
            .map(|n| KeyedUiAccount::from_test_fixtures_json(n).into_keyed_account()),
    )
}
