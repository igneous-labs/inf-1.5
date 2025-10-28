use inf1_core::{
    instructions::swap::exact_in::{
        swap_exact_in_ix_is_signer, swap_exact_in_ix_is_writer, swap_exact_in_ix_keys_owned,
        SwapExactInIxAccs,
    },
    quote::swap::{exact_in::quote_exact_in, SwapQuoteArgs},
};
use inf1_ctl_jiminy::{
    accounts::{lst_state_list::LstStatePackedList, pool_state::PoolStatePacked},
    err::Inf1CtlErr,
    instructions::swap::exact_in::{
        NewSwapExactInIxPreAccsBuilder, SwapExactInIxArgs, SwapExactInIxData,
        SwapExactInIxPreKeysOwned,
    },
    keys::{LST_STATE_LIST_ID, POOL_STATE_ID},
    program_err::Inf1CtlCustomProgErr,
    ID,
};
use inf1_pp_ag_core::{
    inf1_pp_flatslab_core::{
        instructions::pricing::{FlatSlabPpAccs, NewIxSufAccsBuilder},
        keys::SLAB_ID,
    },
    instructions::PriceExactInAccsAg,
    PricingAgTy,
};
use inf1_pp_core::pair::Pair;
use inf1_pp_flatslab_std::FlatSlabPricing;
use inf1_svc_ag_core::{
    calc::SvcCalcAg,
    inf1_svc_lido_core::solido_legacy_core::TOKENKEG_PROGRAM,
    inf1_svc_marinade_core::{
        calc::MarinadeCalc,
        sanctum_marinade_liquid_staking_core::{State, MSOL_MINT_ADDR},
    },
    inf1_svc_spl_core::{calc::SplCalc, sanctum_spl_stake_pool_core::StakePool},
    instructions::SvcCalcAccsAg,
    SvcAg, SvcAgTy,
};
use inf1_svc_jiminy::traits::SolValCalcAccs;
use inf1_test_utils::{
    acc_bef_aft, assert_jiminy_prog_err, find_pool_reserves, find_protocol_fee_accumulator,
    fixtures_accounts_opt_cloned, keys_signer_writable_to_metas, KeyedUiAccount, PkAccountTup,
    JUPSOL_FIXTURE_LST_IDX, JUPSOL_MINT, JUPSOL_POOL_ID, MSOL_FIXTURE_LST_IDX,
};
use mollusk_svm::result::{InstructionResult, ProgramResult};
use sanctum_spl_token_jiminy::sanctum_spl_token_core::state::account::RawTokenAccount;
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

use crate::common::{jupsol_fixtures_svc_suf, msol_fixtures_svc_suf, SVM};

type SwapExactInKeysBuilder = SwapExactInIxAccs<
    [u8; 32],
    SwapExactInIxPreKeysOwned,
    SvcCalcAccsAg,
    SvcCalcAccsAg,
    PriceExactInAccsAg,
>;

fn swap_exact_in_ix_pre_keys_owned(
    signer: [u8; 32],
    inp_token_program: &[u8; 32],
    inp_mint: [u8; 32],
    inp_lst_acc: [u8; 32],
    out_token_program: &[u8; 32],
    out_mint: [u8; 32],
    out_lst_acc: [u8; 32],
) -> SwapExactInIxPreKeysOwned {
    println!(
        "POOL STATE: {:?}",
        Pubkey::new_from_array(POOL_STATE_ID).to_string(),
    );
    println!(
        "bump: {:?}",
        find_protocol_fee_accumulator(out_token_program, &out_mint)
            .0
            .to_string()
    );
    NewSwapExactInIxPreAccsBuilder::start()
        .with_lst_state_list(LST_STATE_LIST_ID)
        .with_pool_state(POOL_STATE_ID)
        .with_inp_lst_mint(inp_mint)
        .with_inp_lst_acc(inp_lst_acc)
        .with_inp_lst_token_program(*inp_token_program)
        .with_out_lst_mint(out_mint)
        .with_out_lst_acc(out_lst_acc)
        .with_out_lst_token_program(*out_token_program)
        .with_protocol_fee_accumulator(
            find_protocol_fee_accumulator(out_token_program, &out_mint)
                .0
                .to_bytes(),
        )
        .with_inp_pool_reserves(
            find_pool_reserves(inp_token_program, &inp_mint)
                .0
                .to_bytes(),
        )
        .with_out_pool_reserves(
            find_pool_reserves(out_token_program, &out_mint)
                .0
                .to_bytes(),
        )
        .with_signer(signer)
        .build()
}

fn swap_exact_in_ix(builder: &SwapExactInKeysBuilder, args: SwapExactInIxArgs) -> Instruction {
    let accounts = keys_signer_writable_to_metas(
        swap_exact_in_ix_keys_owned(builder).seq(),
        swap_exact_in_ix_is_signer(builder).seq(),
        swap_exact_in_ix_is_writer(builder).seq(),
    );

    Instruction {
        program_id: Pubkey::new_from_array(ID),
        accounts,
        data: SwapExactInIxData::new(args).as_buf().into(),
    }
}

fn swap_exact_in_fixtures_accounts_opt(builder: &SwapExactInKeysBuilder) -> Vec<PkAccountTup> {
    fixtures_accounts_opt_cloned(swap_exact_in_ix_keys_owned(builder).seq().copied()).collect()
}

#[test]
fn swap_exact_in_jupsol_msol_fixture() {
    let (jupsol_token_acc_owner_pk, _) =
        KeyedUiAccount::from_test_fixtures_json("jupsol-token-acc-owner.json").into_keyed_account();
    let (jupsol_lst_acc_pk, _) =
        KeyedUiAccount::from_test_fixtures_json("jupsol-token-acc.json").into_keyed_account();

    let (msol_lst_acc_pk, _) =
        KeyedUiAccount::from_test_fixtures_json("msol-token-acc.json").into_keyed_account();

    let ix_prefix = swap_exact_in_ix_pre_keys_owned(
        jupsol_token_acc_owner_pk.to_bytes(),
        &TOKENKEG_PROGRAM,
        JUPSOL_MINT.to_bytes(),
        jupsol_lst_acc_pk.to_bytes(),
        &TOKENKEG_PROGRAM,
        MSOL_MINT_ADDR,
        msol_lst_acc_pk.to_bytes(),
    );

    let builder = SwapExactInKeysBuilder {
        ix_prefix,
        inp_calc_prog: *SvcAgTy::SanctumSplMulti(()).svc_program_id(),
        inp_calc: jupsol_fixtures_svc_suf(),
        out_calc_prog: *SvcAgTy::Marinade(()).svc_program_id(),
        out_calc: msol_fixtures_svc_suf(),
        pricing_prog: *PricingAgTy::FlatSlab(()).program_id(),
        pricing: PriceExactInAccsAg::FlatSlab(FlatSlabPpAccs(
            NewIxSufAccsBuilder::start().with_slab(SLAB_ID).build(),
        )),
    };

    let ix = swap_exact_in_ix(
        &builder,
        SwapExactInIxArgs {
            amount: 10000,
            limit: 8000,
            inp_lst_index: JUPSOL_FIXTURE_LST_IDX as u32,
            out_lst_index: MSOL_FIXTURE_LST_IDX as u32,
            inp_lst_value_calc_accs: jupsol_fixtures_svc_suf().suf_len() + 1,
            out_lst_value_calc_accs: msol_fixtures_svc_suf().suf_len() + 1,
        },
    );
    let accounts = swap_exact_in_fixtures_accounts_opt(&builder);
    let InstructionResult {
        program_result,
        resulting_accounts,
        ..
    } = SVM.with(|svm| svm.process_instruction(&ix, &accounts));

    let [pools, lst_state_lists, inp_lst_accs, out_lst_accs, protocol_fee_accumulator_accs, out_pool_reserves_accs] =
        [
            POOL_STATE_ID,
            LST_STATE_LIST_ID,
            *ix_prefix.inp_lst_acc(),
            *ix_prefix.out_lst_acc(),
            *ix_prefix.protocol_fee_accumulator(),
            *ix_prefix.out_pool_reserves(),
        ]
        .map(|a| acc_bef_aft(&Pubkey::new_from_array(a), &accounts, &resulting_accounts));

    let [pool_bef, pool_aft] = pools.each_ref().map(|a| {
        PoolStatePacked::of_acc_data(&a.data)
            .unwrap()
            .into_pool_state()
    });
    let [lst_state_list_bef, lst_state_list_aft] = lst_state_lists
        .each_ref()
        .map(|a| LstStatePackedList::of_acc_data(&a.data).unwrap());

    let [inp_lst_acc_bef, inp_lst_acc_aft] = inp_lst_accs
        .each_ref()
        .map(|a| RawTokenAccount::of_acc_data(&a.data).unwrap());
    let [out_lst_acc_bef, out_lst_acc_aft] = out_lst_accs
        .each_ref()
        .map(|a| RawTokenAccount::of_acc_data(&a.data).unwrap());
    let [protocol_fee_accumulator_bef, protocol_fee_accumulator_aft] =
        protocol_fee_accumulator_accs
            .each_ref()
            .map(|a| RawTokenAccount::of_acc_data(&a.data).unwrap());

    let [out_pool_reserves_bef, _] = out_pool_reserves_accs
        .each_ref()
        .map(|a| RawTokenAccount::of_acc_data(&a.data).unwrap());

    let (_, jupsol_pool_acc) =
        KeyedUiAccount::from_test_fixtures_json("jupsol-pool.json").into_keyed_account();
    let (_, marinade_pool_acc) =
        KeyedUiAccount::from_test_fixtures_json("msol-pool.json").into_keyed_account();
    let (_, slab_acc) =
        KeyedUiAccount::from_test_fixtures_json("flatslab-slab.json").into_keyed_account();

    let jupsol_stakepool = StakePool::borsh_de(jupsol_pool_acc.data.as_slice()).unwrap();
    let marinade_stakepool = State::borsh_de(marinade_pool_acc.data.as_slice()).unwrap();

    let quote = quote_exact_in(SwapQuoteArgs {
        amt: 10000,
        out_reserves: u64::from_le_bytes(out_pool_reserves_bef.amount),
        trading_protocol_fee_bps: pool_bef.trading_protocol_fee_bps,
        inp_mint: JUPSOL_MINT.to_bytes(),
        out_mint: MSOL_MINT_ADDR,
        inp_calc: SplCalc::new(&jupsol_stakepool, 0),
        out_calc: MarinadeCalc::new(&marinade_stakepool),
        pricing: FlatSlabPricing::new(slab_acc.data.into_boxed_slice())
            .flat_slab_swap_pricing_for(&Pair {
                inp: &JUPSOL_MINT.to_bytes(),
                out: &MSOL_MINT_ADDR,
            })
            .unwrap(),
    })
    .unwrap();

    let inp_lst_state_idx = lst_state_list_bef
        .0
        .iter()
        .position(|s| s.into_lst_state().mint == JUPSOL_MINT.to_bytes())
        .unwrap();

    let out_lst_state_idx = lst_state_list_aft
        .0
        .iter()
        .position(|s| s.into_lst_state().mint == MSOL_MINT_ADDR)
        .unwrap();

    let [inp_lst_state_bef, inp_lst_state_aft] =
        [lst_state_list_bef, lst_state_list_aft].map(|l| l.0[inp_lst_state_idx].into_lst_state());
    let [out_lst_state_bef, out_lst_state_aft] =
        [lst_state_list_bef, lst_state_list_aft].map(|l| l.0[out_lst_state_idx].into_lst_state());

    let inp_sol_val_delta =
        i128::from(inp_lst_state_aft.sol_value) - i128::from(inp_lst_state_bef.sol_value);
    let out_sol_val_delta =
        i128::from(out_lst_state_aft.sol_value) - i128::from(out_lst_state_bef.sol_value);

    let total_delta = inp_sol_val_delta + out_sol_val_delta;

    let pool_sol_val_delta =
        i128::from(pool_aft.total_sol_value) - i128::from(pool_bef.total_sol_value);

    assert_eq!(total_delta, pool_sol_val_delta);
    assert!(pool_sol_val_delta > 0);

    let inp_lst_balance_delta =
        u64::from_le_bytes(inp_lst_acc_bef.amount) - u64::from_le_bytes(inp_lst_acc_aft.amount);
    let out_lst_balance_delta =
        u64::from_le_bytes(out_lst_acc_aft.amount) - u64::from_le_bytes(out_lst_acc_bef.amount);
    let protocol_fee_accumulator_balance_delta =
        u64::from_le_bytes(protocol_fee_accumulator_aft.amount)
            - u64::from_le_bytes(protocol_fee_accumulator_bef.amount);

    assert_eq!(inp_lst_balance_delta, quote.0.inp);
    assert_eq!(out_lst_balance_delta, quote.0.out);
    assert_eq!(protocol_fee_accumulator_balance_delta, quote.0.protocol_fee);

    println!("quoteyyyy: {:?}", quote.0.out);

    assert_eq!(program_result, ProgramResult::Success);
}
