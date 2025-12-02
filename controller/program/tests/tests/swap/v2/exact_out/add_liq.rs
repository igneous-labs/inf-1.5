use std::collections::HashMap;

use expect_test::expect;
use inf1_ctl_jiminy::{
    instructions::swap::v2::{exact_out::NewSwapExactOutV2IxPreAccsBuilder, IxPreAccs},
    svc::{InfCalc, InfDummyCalcAccs},
};
use inf1_pp_ag_core::{PricingAg, PricingAgTy};
use inf1_pp_core::pair::Pair;
use inf1_pp_flatslab_std::accounts::Slab;
use inf1_std::quote::swap::{exact_out::quote_exact_out, QuoteArgs};
use inf1_svc_ag_core::{
    calc::SvcCalcAg,
    inf1_svc_spl_core::{calc::SplCalc, sanctum_spl_stake_pool_core::StakePool},
    instructions::SvcCalcAccsAg,
    SvcAg, SvcAgTy,
};
use inf1_test_utils::{
    flatslab_fixture_suf_accs, get_lst_state_list, get_mint_suppply, get_token_account_amount,
    jupsol_fixture_svc_suf_accs, mollusk_exec, KeyedUiAccount, VerPoolState,
    JUPSOL_FIXTURE_LST_IDX,
};
use mollusk_svm::result::{InstructionResult, ProgramResult};

use crate::{
    common::{header_lookahead, Cbs, SVM},
    tests::swap::common::assert_swap_token_movements,
};

use super::{add_prog_accs, to_ix, Accs, Args};

#[test]
fn swap_exact_out_v2_jupsol_add_liq_fixture() {
    let migration_slot = 0;
    let amount = 10_000;
    let prefix_am = NewSwapExactOutV2IxPreAccsBuilder::start()
        .with_signer("jupsol-token-acc-owner")
        .with_pool_state("pool-state")
        .with_lst_state_list("lst-state-list")
        .with_inp_acc("jupsol-token-acc")
        .with_inp_mint("jupsol-mint")
        .with_inp_pool_reserves("jupsol-reserves")
        .with_out_acc("inf-token-acc")
        .with_out_mint("inf-mint")
        .with_out_pool_reserves("inf-mint")
        .with_inp_token_program("tokenkeg")
        .with_out_token_program("tokenkeg")
        .build()
        .0
        .map(|n| KeyedUiAccount::from_test_fixtures_json(n).into_keyed_account());
    let prefix_keys = IxPreAccs(prefix_am.each_ref().map(|(addr, _)| addr.to_bytes()));
    let prefix_refs = IxPreAccs(prefix_am.each_ref());
    let (pp_accs, pp_am) = flatslab_fixture_suf_accs();
    let (inp_accs, inp_am) = jupsol_fixture_svc_suf_accs();

    let inp_calc = SplCalc::new(
        &StakePool::borsh_de(inp_am[&inp_accs.stake_pool_addr.into()].data.as_slice()).unwrap(),
        0,
    );
    let ps = VerPoolState::from_acc_data(&prefix_refs.pool_state().1.data).migrated(migration_slot);
    let ps = header_lookahead(
        ps,
        &[Cbs {
            calc: &inp_calc,
            balance: get_token_account_amount(&prefix_refs.inp_pool_reserves().1.data),
            old_sol_val: get_lst_state_list(&prefix_refs.lst_state_list().1.data)
                [JUPSOL_FIXTURE_LST_IDX]
                .sol_value,
        }],
        migration_slot,
    );
    let out_calc = SvcCalcAg::Inf(InfCalc::new(
        &ps,
        get_mint_suppply(&prefix_refs.out_mint().1.data),
    ));
    let pricing = Slab::of_acc_data(&pp_am[&(*pp_accs.0.slab()).into()].data)
        .unwrap()
        .entries()
        .pricing(&Pair {
            inp: prefix_refs.inp_mint().0.as_array(),
            out: prefix_refs.out_mint().0.as_array(),
        })
        .unwrap();

    let quote = quote_exact_out(&QuoteArgs {
        amt: amount,
        inp_mint: prefix_refs.inp_mint().0.to_bytes(),
        out_mint: prefix_refs.out_mint().0.to_bytes(),
        inp_calc,
        out_calc,
        pricing,
        out_reserves: u64::MAX,
    })
    .unwrap();

    let accs = Accs {
        ix_prefix: prefix_keys,
        inp_calc_prog: *SvcAgTy::SanctumSplMulti(()).svc_program_id(),
        inp_calc: SvcAg::SanctumSplMulti(inp_accs),
        out_calc_prog: Default::default(), // dont-care, filler
        out_calc: SvcCalcAccsAg::Inf(InfDummyCalcAccs),
        pricing_prog: *PricingAgTy::FlatSlab(()).program_id(),
        pricing: PricingAg::FlatSlab(pp_accs),
    };
    let mut bef = prefix_am.into_iter().chain(pp_am).chain(inp_am).collect();
    add_prog_accs(&mut bef, &accs);
    let args = Args {
        inp_lst_index: JUPSOL_FIXTURE_LST_IDX.try_into().unwrap(),
        out_lst_index: u32::MAX,
        limit: u64::MAX,
        amount,
        accs,
    };
    let ix = to_ix(&args);

    let (
        _,
        InstructionResult {
            program_result,
            resulting_accounts,
            ..
        },
    ) = SVM.with(|svm| mollusk_exec(svm, &ix, &bef));
    let aft: HashMap<_, _> = resulting_accounts.into_iter().collect();

    assert_eq!(program_result, ProgramResult::Success);
    assert_swap_token_movements(&bef, &aft, &prefix_keys, &quote);
    expect![[r#"
        (
            12049,
            10000,
            121,
        )
    "#]]
    .assert_debug_eq(&(quote.inp, quote.out, quote.fee));
}
