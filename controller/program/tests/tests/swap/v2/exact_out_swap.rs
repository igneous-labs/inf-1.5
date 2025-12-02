use std::collections::HashMap;

use expect_test::expect;
use inf1_ctl_jiminy::{
    instructions::swap::v2::{
        exact_out::{NewSwapExactOutV2IxPreAccsBuilder, SwapExactOutIxData},
        IxPreAccs,
    },
    ID,
};
use inf1_pp_ag_core::{instructions::PriceExactOutAccsAg, PricingAg, PricingAgTy};
use inf1_pp_core::pair::Pair;
use inf1_pp_flatslab_std::accounts::Slab;
use inf1_std::{
    instructions::swap::v2::exact_out::{
        swap_exact_out_v2_ix_is_signer, swap_exact_out_v2_ix_is_writer,
        swap_exact_out_v2_ix_keys_owned,
    },
    quote::swap::{exact_out::quote_exact_out, QuoteArgs},
};
use inf1_svc_ag_core::{
    calc::SvcCalcAg,
    inf1_svc_spl_core::{calc::SplCalc, sanctum_spl_stake_pool_core::StakePool},
    inf1_svc_wsol_core::{calc::WsolCalc, instructions::sol_val_calc::WsolCalcAccs},
    instructions::SvcCalcAccsAg,
    SvcAg, SvcAgTy,
};
use inf1_test_utils::{
    flatslab_fixture_suf_accs, get_token_account_amount, jupsol_fixture_svc_suf_accs,
    keys_signer_writable_to_metas, mock_prog_acc, mollusk_exec, AccountMap, KeyedUiAccount,
    ProgramDataAddr, JUPSOL_FIXTURE_LST_IDX, JUPSOL_MINT, WSOL_FIXTURE_LST_IDX, WSOL_MINT,
};
use mollusk_svm::result::{InstructionResult, ProgramResult};
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

use crate::{common::SVM, tests::swap::common::assert_swap_token_movements};

type Accs = super::Accs<PriceExactOutAccsAg>;
type Args = super::Args<PriceExactOutAccsAg>;

fn to_ix(args: &Args) -> Instruction {
    let accounts = keys_signer_writable_to_metas(
        swap_exact_out_v2_ix_keys_owned(&args.accs).seq(),
        swap_exact_out_v2_ix_is_signer(&args.accs).seq(),
        swap_exact_out_v2_ix_is_writer(&args.accs).seq(),
    );
    Instruction {
        program_id: Pubkey::new_from_array(ID),
        accounts,
        data: SwapExactOutIxData::new(&args.to_full()).as_buf().into(),
    }
}

fn add_prog_accs(
    am: &mut AccountMap,
    Accs {
        inp_calc_prog,
        out_calc_prog,
        pricing_prog,
        ..
    }: &Accs,
) {
    am.extend(
        [inp_calc_prog, out_calc_prog, pricing_prog]
            .into_iter()
            .map(|addr| {
                (
                    Pubkey::new_from_array(*addr),
                    // dont-care
                    mock_prog_acc(ProgramDataAddr::Raw(Default::default())),
                )
            }),
    );
}

#[test]
fn swap_exact_out_v2_jupsol_to_wsol_fixture() {
    let amount = 10_000;
    let prefix_am = NewSwapExactOutV2IxPreAccsBuilder::start()
        .with_signer("jupsol-token-acc-owner")
        .with_pool_state("pool-state")
        .with_lst_state_list("lst-state-list")
        .with_inp_acc("jupsol-token-acc")
        .with_inp_mint("jupsol-mint")
        .with_inp_pool_reserves("jupsol-reserves")
        .with_out_acc("wsol-token-acc")
        .with_out_mint("wsol-mint")
        .with_out_pool_reserves("wsol-reserves")
        .with_inp_token_program("tokenkeg")
        .with_out_token_program("tokenkeg")
        .build()
        .0
        .map(|n| KeyedUiAccount::from_test_fixtures_json(n).into_keyed_account());
    let prefix_keys = IxPreAccs(prefix_am.each_ref().map(|(addr, _)| addr.to_bytes()));
    let out_accs = SvcCalcAccsAg::Wsol(WsolCalcAccs);
    let (pp_accs, pp_am) = flatslab_fixture_suf_accs();
    let (inp_accs, inp_am) = jupsol_fixture_svc_suf_accs();

    let out_calc = SvcCalcAg::Wsol(WsolCalc);
    let pricing = Slab::of_acc_data(&pp_am[&(*pp_accs.0.slab()).into()].data)
        .unwrap()
        .entries()
        .pricing(&Pair {
            inp: JUPSOL_MINT.as_array(),
            out: WSOL_MINT.as_array(),
        })
        .unwrap();
    let inp_calc = SplCalc::new(
        &StakePool::borsh_de(inp_am[&inp_accs.stake_pool_addr.into()].data.as_slice()).unwrap(),
        0,
    );

    let quote = quote_exact_out(&QuoteArgs {
        amt: amount,
        inp_mint: JUPSOL_MINT.to_bytes(),
        out_mint: WSOL_MINT.to_bytes(),
        inp_calc,
        out_calc,
        pricing,
        out_reserves: get_token_account_amount(
            &IxPreAccs(prefix_am.each_ref()).out_pool_reserves().1.data,
        ),
    })
    .unwrap();

    let accs = Accs {
        ix_prefix: prefix_keys,
        inp_calc_prog: *SvcAgTy::SanctumSplMulti(()).svc_program_id(),
        inp_calc: SvcAg::SanctumSplMulti(inp_accs),
        out_calc_prog: *SvcAgTy::Wsol(()).svc_program_id(),
        out_calc: out_accs,
        pricing_prog: *PricingAgTy::FlatSlab(()).program_id(),
        pricing: PricingAg::FlatSlab(pp_accs),
    };
    let mut bef = prefix_am.into_iter().chain(pp_am).chain(inp_am).collect();
    add_prog_accs(&mut bef, &accs);
    let args = Args {
        inp_lst_index: JUPSOL_FIXTURE_LST_IDX.try_into().unwrap(),
        out_lst_index: WSOL_FIXTURE_LST_IDX.try_into().unwrap(),
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
            9031,
            10000,
            51,
        )
    "#]]
    .assert_debug_eq(&(quote.inp, quote.out, quote.fee));
}
