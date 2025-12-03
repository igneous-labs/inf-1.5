use expect_test::expect;
use inf1_ctl_jiminy::instructions::swap::v1::{exact_in::SwapExactInIxData, IxPreAccs};
use inf1_pp_ag_core::{instructions::PriceExactInAccsAg, PricingAgTy};
use inf1_std::{
    instructions::swap::v1::exact_in::{
        swap_exact_in_ix_is_signer, swap_exact_in_ix_is_writer, swap_exact_in_ix_keys_owned,
    },
    quote::Quote,
};
use inf1_svc_ag_core::{SvcAg, SvcAgTy};
use inf1_test_utils::{
    assert_jiminy_prog_err, flatslab_fixture_suf_accs, jupsol_fixture_svc_suf_accs,
    keys_signer_writable_to_metas, mollusk_exec, msol_fixture_svc_suf_accs, AccountMap,
    JUPSOL_FIXTURE_LST_IDX, MSOL_FIXTURE_LST_IDX,
};
use jiminy_cpi::program_error::ProgramError;
use mollusk_svm::Mollusk;
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

use crate::{
    common::SVM,
    tests::swap::{
        common::{add_swap_prog_accs, assert_correct_swap_exact_in_v2},
        v1::{args_to_v2, jupsol_to_msol_prefix_fixtures},
        V1Accs, V1Args,
    },
};

type Args = V1Args<PriceExactInAccsAg>;

fn to_ix(args: &Args) -> Instruction {
    let accounts = keys_signer_writable_to_metas(
        swap_exact_in_ix_keys_owned(&args.accs).seq(),
        swap_exact_in_ix_is_signer(&args.accs).seq(),
        swap_exact_in_ix_is_writer(&args.accs).seq(),
    );
    Instruction {
        program_id: Pubkey::new_from_array(inf1_ctl_jiminy::ID),
        accounts,
        data: SwapExactInIxData::new(&args.to_full()).as_buf().into(),
    }
}

/// Returns `None` if expected_err is `Some`
fn swap_exact_in_test(
    svm: &Mollusk,
    args: &Args,
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

#[test]
fn swap_exact_in_jupsol_to_msol_fixture() {
    const AMOUNT: u64 = 8_000;

    let prefix_am = jupsol_to_msol_prefix_fixtures();
    let prefix_keys = IxPreAccs(prefix_am.0.each_ref().map(|(addr, _)| addr.to_bytes()));
    let (pp_accs, pp_am) = flatslab_fixture_suf_accs();
    let (inp_accs, inp_am) = jupsol_fixture_svc_suf_accs();
    let (out_accs, out_am) = msol_fixture_svc_suf_accs();

    let accs = V1Accs {
        ix_prefix: prefix_keys,
        inp_calc_prog: *SvcAgTy::SanctumSplMulti(()).svc_program_id(),
        inp_calc: SvcAg::SanctumSplMulti(inp_accs),
        out_calc_prog: *SvcAgTy::Marinade(()).svc_program_id(),
        out_calc: SvcAg::Marinade(out_accs),
        pricing_prog: *PricingAgTy::FlatSlab(()).program_id(),
        pricing: PriceExactInAccsAg::FlatSlab(pp_accs),
    };
    let args = V1Args {
        inp_lst_index: JUPSOL_FIXTURE_LST_IDX.try_into().unwrap(),
        out_lst_index: MSOL_FIXTURE_LST_IDX.try_into().unwrap(),
        limit: 0,
        amount: AMOUNT,
        accs,
    };

    let mut bef = prefix_am
        .0
        .into_iter()
        .chain(pp_am)
        .chain(inp_am)
        .chain(out_am)
        .collect();
    add_swap_prog_accs(&mut bef, &accs);

    let Quote { inp, out, fee, .. } =
        SVM.with(|svm| swap_exact_in_test(svm, &args, &bef, None::<ProgramError>).unwrap());

    expect![[r#"
        (
            8000,
            6842,
            27,
        )
    "#]]
    .assert_debug_eq(&(inp, out, fee));
}
