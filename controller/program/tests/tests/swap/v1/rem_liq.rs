#![allow(deprecated)]

use expect_test::expect;
use inf1_ctl_jiminy::{
    instructions::{
        liquidity::{remove::RemoveLiquidityIxData, IxPreAccs, NewIxPreAccsBuilder},
        swap::v2,
    },
    svc::InfDummyCalcAccs,
};
use inf1_pp_ag_core::{instructions::PriceExactInAccsAg, PricingAgTy};
use inf1_std::{
    instructions::{
        liquidity::remove::{
            remove_liquidity_ix_is_signer, remove_liquidity_ix_is_writer,
            remove_liquidity_ix_keys_owned,
        },
        swap::IxAccs,
    },
    quote::Quote,
};
use inf1_svc_ag_core::{SvcAg, SvcAgTy};
use inf1_test_utils::{
    assert_jiminy_prog_err, flatslab_fixture_suf_accs, jupsol_fixture_svc_suf_accs,
    keys_signer_writable_to_metas, mollusk_exec, AccountMap, KeyedUiAccount,
    JUPSOL_FIXTURE_LST_IDX,
};
use jiminy_cpi::program_error::ProgramError;
use mollusk_svm::Mollusk;
use solana_account::Account;
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

use crate::{
    common::SVM,
    tests::swap::{
        common::assert_correct_swap_exact_in_v2, v1::fill_liq_prog_accs, LiqAccs, LiqArgs, V2Args,
    },
};

fn args_to_v2(
    LiqArgs {
        lst_index,
        amount,
        min_out,
        accs:
            LiqAccs {
                ix_prefix,
                lst_calc_prog,
                lst_calc,
                pricing_prog,
                pricing,
            },
    }: LiqArgs,
) -> V2Args {
    V2Args {
        inp_lst_index: u32::MAX,
        out_lst_index: lst_index,
        limit: min_out,
        amount,
        accs: IxAccs {
            ix_prefix: v2::IxPreAccs::clone_from_rem_liq(&ix_prefix),
            inp_calc_prog: inf1_ctl_jiminy::ID,
            inp_calc: SvcAg::Inf(InfDummyCalcAccs),
            out_calc_prog: lst_calc_prog,
            out_calc: lst_calc,
            pricing_prog,
            pricing: PriceExactInAccsAg::FlatSlab(pricing),
        },
    }
}

fn to_ix(args: &LiqArgs) -> Instruction {
    let accounts = keys_signer_writable_to_metas(
        remove_liquidity_ix_keys_owned(&args.accs).seq(),
        remove_liquidity_ix_is_signer(&args.accs).seq(),
        remove_liquidity_ix_is_writer(&args.accs).seq(),
    );
    Instruction {
        program_id: Pubkey::new_from_array(inf1_ctl_jiminy::ID),
        accounts,
        data: RemoveLiquidityIxData::new(&args.to_full()).as_buf().into(),
    }
}

/// Returns `None` if expected_err is `Some`
fn rem_liq_test(
    svm: &Mollusk,
    args: &LiqArgs,
    bef: &AccountMap,
    expected_err: Option<impl Into<ProgramError>>,
) -> Option<Quote> {
    let ix = to_ix(args);

    let result = mollusk_exec(svm, &[ix], bef);

    match expected_err {
        None => {
            let aft = result.unwrap().resulting_accounts;
            let clock = &svm.sysvars.clock;
            Some(assert_correct_swap_exact_in_v2(
                bef,
                &aft,
                &args_to_v2(*args),
                clock.epoch,
                clock.slot,
            ))
        }
        Some(e) => {
            assert_jiminy_prog_err(&result.unwrap_err(), e);
            None
        }
    }
}

fn jupsol_rem_liq_fixtures() -> IxPreAccs<(Pubkey, Account)> {
    IxPreAccs(
        NewIxPreAccsBuilder::start()
            .with_signer("inf-token-acc-owner")
            .with_pool_state("pool-state")
            .with_lst_state_list("lst-state-list")
            .with_lst_acc("jupsol-token-acc")
            .with_lst_mint("jupsol-mint")
            .with_pool_reserves("jupsol-reserves")
            .with_lp_acc("inf-token-acc")
            .with_lp_token_mint("inf-mint")
            .with_protocol_fee_accumulator("jupsol-pf-accum")
            // filler
            .with_lst_token_program("inf-mint")
            .with_lp_token_program("inf-mint")
            .build()
            .0
            .map(|n| KeyedUiAccount::from_test_fixtures_json(n).into_keyed_account()),
    )
    .with_lst_token_program(mollusk_svm_programs_token::token::keyed_account())
    .with_lp_token_program(mollusk_svm_programs_token::token::keyed_account())
}

#[test]
fn rem_liq_jupsol_fixture() {
    let amount = 10_000;

    let prefix_am = jupsol_rem_liq_fixtures();
    let prefix_keys = IxPreAccs(prefix_am.0.each_ref().map(|(addr, _)| addr.to_bytes()));
    let (pp_accs, pp_am) = flatslab_fixture_suf_accs();
    let (inp_accs, inp_am) = jupsol_fixture_svc_suf_accs();

    let accs = LiqAccs {
        ix_prefix: prefix_keys,
        lst_calc_prog: *SvcAgTy::SanctumSplMulti(()).svc_program_id(),
        lst_calc: SvcAg::SanctumSplMulti(inp_accs),
        pricing_prog: *PricingAgTy::FlatSlab(()).program_id(),
        pricing: pp_accs,
    };
    let args = LiqArgs {
        lst_index: JUPSOL_FIXTURE_LST_IDX.try_into().unwrap(),
        amount,
        min_out: 0,
        accs,
    };

    let mut bef = prefix_am.0.into_iter().chain(pp_am).chain(inp_am).collect();
    fill_liq_prog_accs(&mut bef, &accs);

    let Quote { inp, out, fee, .. } =
        SVM.with(|svm| rem_liq_test(svm, &args, &bef, None::<ProgramError>).unwrap());

    expect![[r#"
        (
            10000,
            19877,
            157,
        )
    "#]]
    .assert_debug_eq(&(inp, out, fee));
}
