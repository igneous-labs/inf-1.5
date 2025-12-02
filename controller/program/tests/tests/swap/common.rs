use std::ops::Neg;

use inf1_ctl_jiminy::{accounts::pool_state::PoolStateV2Packed, instructions::swap::v2::IxPreAccs};
use inf1_std::quote::Quote;
use inf1_test_utils::{
    acc_bef_aft, assert_token_acc_diffs, get_mint_suppply, token_acc_bal_diff_changed, AccountMap,
    Diff,
};
use sanctum_spl_token_jiminy::sanctum_spl_token_core::state::account::RawTokenAccount;
use solana_pubkey::Pubkey;

// TODO: need to assert more things beyond token changes,
// but that requires lookahead of update_yield and release_yield

pub fn assert_swap_token_movements(
    bef: &AccountMap,
    aft: &AccountMap,
    accs: &IxPreAccs<impl Into<Pubkey> + Copy>,
    quote: &Quote,
) {
    let Quote {
        inp,
        out,
        inp_mint,
        out_mint,
        ..
    } = quote;

    // user's token accs
    let [user_inp, user_out] = [accs.inp_acc(), accs.out_acc()].map(|addr| {
        acc_bef_aft(&(*addr).into(), bef, aft)
            .map(|a| RawTokenAccount::of_acc_data(&a.data).unwrap())
    });
    [
        (user_inp, i128::from(*inp).neg()),
        (user_out, i128::from(*out)),
    ]
    .into_iter()
    .for_each(|([a_bef, a_aft], change)| {
        assert_token_acc_diffs(a_bef, a_aft, &token_acc_bal_diff_changed(a_bef, change))
    });

    let lp_mint =
        PoolStateV2Packed::of_acc_data(&aft.get(&(*accs.pool_state()).into()).unwrap().data)
            .unwrap()
            .into_pool_state_v2()
            .lp_token_mint;

    if *inp_mint == lp_mint {
        assert_pool_token_movements_rem_liq(bef, aft, accs, quote);
    } else if *out_mint == lp_mint {
        assert_pool_token_movements_add_liq(bef, aft, accs, quote);
    } else {
        assert_pool_token_movements_swap(bef, aft, accs, quote);
    }
}

fn assert_pool_token_movements_swap(
    bef: &AccountMap,
    aft: &AccountMap,
    accs: &IxPreAccs<impl Into<Pubkey> + Copy>,
    Quote { inp, out, .. }: &Quote,
) {
    let [inp_reserves, out_reserves] =
        [accs.inp_pool_reserves(), accs.out_pool_reserves()].map(|addr| {
            acc_bef_aft(&(*addr).into(), bef, aft)
                .map(|a| RawTokenAccount::of_acc_data(&a.data).unwrap())
        });
    [
        (inp_reserves, i128::from(*inp)),
        (out_reserves, i128::from(*out).neg()),
    ]
    .into_iter()
    .for_each(|([a_bef, a_aft], change)| {
        assert_token_acc_diffs(a_bef, a_aft, &token_acc_bal_diff_changed(a_bef, change))
    });
}

fn assert_pool_token_movements_add_liq(
    bef: &AccountMap,
    aft: &AccountMap,
    accs: &IxPreAccs<impl Into<Pubkey> + Copy>,
    Quote { inp, out, .. }: &Quote,
) {
    let [inp_reserves_bef, inp_reserves_aft] =
        acc_bef_aft(&(*accs.inp_pool_reserves()).into(), bef, aft)
            .map(|a| RawTokenAccount::of_acc_data(&a.data).unwrap());
    assert_token_acc_diffs(
        inp_reserves_bef,
        inp_reserves_aft,
        &token_acc_bal_diff_changed(inp_reserves_bef, i128::from(*inp)),
    );
    let [lp_supp_bef, lp_supp_aft] =
        acc_bef_aft(&(*accs.out_mint()).into(), bef, aft).map(|a| get_mint_suppply(&a.data));
    Diff::Changed(lp_supp_bef, lp_supp_bef + out).assert(&lp_supp_bef, &lp_supp_aft);
}

fn assert_pool_token_movements_rem_liq(
    bef: &AccountMap,
    aft: &AccountMap,
    accs: &IxPreAccs<impl Into<Pubkey> + Copy>,
    Quote { inp, out, .. }: &Quote,
) {
    let [lp_supp_bef, lp_supp_aft] =
        acc_bef_aft(&(*accs.inp_mint()).into(), bef, aft).map(|a| get_mint_suppply(&a.data));
    Diff::Changed(lp_supp_bef, lp_supp_bef - inp).assert(&lp_supp_bef, &lp_supp_aft);
    let [out_reserves_bef, out_reserves_aft] =
        acc_bef_aft(&(*accs.out_pool_reserves()).into(), bef, aft)
            .map(|a| RawTokenAccount::of_acc_data(&a.data).unwrap());
    assert_token_acc_diffs(
        out_reserves_bef,
        out_reserves_aft,
        &token_acc_bal_diff_changed(out_reserves_bef, i128::from(*out).neg()),
    );
}
