use inf1_core::{
    instructions::swap::{
        exact_in::{
            swap_exact_in_ix_is_signer, swap_exact_in_ix_is_writer, swap_exact_in_ix_keys_owned,
        },
        IxAccs,
    },
    quote::swap::{exact_in::quote_exact_in, exact_out::quote_exact_out, SwapQuoteArgs},
};
use inf1_ctl_jiminy::{
    accounts::{lst_state_list::LstStatePackedList, pool_state::PoolStatePacked},
    instructions::swap::{
        exact_in::{SwapExactInIxData, SwapExactInIxPreKeysOwned},
        exact_out::SwapExactOutIxData,
        IxArgs, IxPreKeysOwned, NewIxPreAccsBuilder,
    },
    keys::{LST_STATE_LIST_ID, POOL_STATE_ID},
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
use inf1_pp_core::{
    pair::Pair,
    traits::main::{PriceExactIn, PriceExactOut},
};
use inf1_pp_flatslab_std::FlatSlabPricing;
use inf1_svc_ag_core::{
    inf1_svc_lido_core::solido_legacy_core::TOKENKEG_PROGRAM,
    inf1_svc_marinade_core::{
        calc::MarinadeCalc,
        sanctum_marinade_liquid_staking_core::{State, MSOL_MINT_ADDR},
    },
    inf1_svc_spl_core::{calc::SplCalc, sanctum_spl_stake_pool_core::StakePool},
    instructions::SvcCalcAccsAg,
    SvcAgTy,
};
use inf1_svc_jiminy::traits::{SolValCalc, SolValCalcAccs};
use inf1_test_utils::{
    acc_bef_aft, find_pool_reserves_ata, find_protocol_fee_accumulator_ata,
    fixtures_accounts_opt_cloned, keys_signer_writable_to_metas, KeyedUiAccount, PkAccountTup,
    JUPSOL_FIXTURE_LST_IDX, JUPSOL_MINT, MSOL_FIXTURE_LST_IDX,
};
use sanctum_spl_token_jiminy::sanctum_spl_token_core::state::account::RawTokenAccount;
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

use crate::common::{jupsol_fixtures_svc_suf, msol_fixtures_svc_suf};

#[derive(Debug, Clone, Copy)]
pub enum SwapIxType {
    ExactIn,
    ExactOut,
}

pub type SwapKeysBuilder =
    IxAccs<[u8; 32], SwapExactInIxPreKeysOwned, SvcCalcAccsAg, SvcCalcAccsAg, PriceExactInAccsAg>;

pub fn get_swap_ix(builder: &SwapKeysBuilder, args: IxArgs, ix_type: SwapIxType) -> Instruction {
    let accounts = keys_signer_writable_to_metas(
        swap_exact_in_ix_keys_owned(builder).seq(),
        swap_exact_in_ix_is_signer(builder).seq(),
        swap_exact_in_ix_is_writer(builder).seq(),
    );

    Instruction {
        program_id: Pubkey::new_from_array(ID),
        accounts,
        data: match ix_type {
            SwapIxType::ExactIn => SwapExactInIxData::new(args).as_buf().into(),
            SwapIxType::ExactOut => SwapExactOutIxData::new(args).as_buf().into(),
        },
    }
}

pub fn swap_ix_pre_keys_owned(
    signer: [u8; 32],
    inp_token_program: &[u8; 32],
    inp_mint: [u8; 32],
    inp_lst_acc: [u8; 32],
    out_token_program: &[u8; 32],
    out_mint: [u8; 32],
    out_lst_acc: [u8; 32],
) -> IxPreKeysOwned {
    NewIxPreAccsBuilder::start()
        .with_lst_state_list(LST_STATE_LIST_ID)
        .with_pool_state(POOL_STATE_ID)
        .with_inp_lst_mint(inp_mint)
        .with_inp_lst_acc(inp_lst_acc)
        .with_inp_lst_token_program(*inp_token_program)
        .with_out_lst_mint(out_mint)
        .with_out_lst_acc(out_lst_acc)
        .with_out_lst_token_program(*out_token_program)
        .with_protocol_fee_accumulator(
            find_protocol_fee_accumulator_ata(out_token_program, &out_mint)
                .0
                .to_bytes(),
        )
        .with_inp_pool_reserves(
            find_pool_reserves_ata(inp_token_program, &inp_mint)
                .0
                .to_bytes(),
        )
        .with_out_pool_reserves(
            find_pool_reserves_ata(out_token_program, &out_mint)
                .0
                .to_bytes(),
        )
        .with_signer(signer)
        .build()
}

pub fn get_jupsol_msol_setup(
    amount: u64,
    limit: u64,
    ix_type: SwapIxType,
) -> (
    SwapExactInIxPreKeysOwned,
    Instruction,
    SwapKeysBuilder,
    impl SolValCalc,
    impl SolValCalc,
    impl PriceExactIn + PriceExactOut,
) {
    let (jupsol_token_acc_owner_pk, _) =
        KeyedUiAccount::from_test_fixtures_json("jupsol-token-acc-owner.json").into_keyed_account();
    let (jupsol_lst_acc_pk, _) =
        KeyedUiAccount::from_test_fixtures_json("jupsol-token-acc.json").into_keyed_account();

    let (msol_lst_acc_pk, _) =
        KeyedUiAccount::from_test_fixtures_json("msol-token-acc.json").into_keyed_account();

    let (_, jupsol_pool_acc) =
        KeyedUiAccount::from_test_fixtures_json("jupsol-pool.json").into_keyed_account();
    let (_, marinade_pool_acc) =
        KeyedUiAccount::from_test_fixtures_json("msol-pool.json").into_keyed_account();
    let (_, slab_acc) =
        KeyedUiAccount::from_test_fixtures_json("flatslab-slab.json").into_keyed_account();

    let jupsol_stakepool = StakePool::borsh_de(jupsol_pool_acc.data.as_slice()).unwrap();
    let marinade_stakepool = State::borsh_de(marinade_pool_acc.data.as_slice()).unwrap();

    let inp_calc = SplCalc::new(&jupsol_stakepool, 0);
    let out_calc = MarinadeCalc::new(&marinade_stakepool);
    let pricing = FlatSlabPricing::new(slab_acc.data.into_boxed_slice())
        .flat_slab_swap_pricing_for(&Pair {
            inp: &JUPSOL_MINT.to_bytes(),
            out: &MSOL_MINT_ADDR,
        })
        .unwrap();

    let ix_prefix = swap_ix_pre_keys_owned(
        jupsol_token_acc_owner_pk.to_bytes(),
        &TOKENKEG_PROGRAM,
        JUPSOL_MINT.to_bytes(),
        jupsol_lst_acc_pk.to_bytes(),
        &TOKENKEG_PROGRAM,
        MSOL_MINT_ADDR,
        msol_lst_acc_pk.to_bytes(),
    );

    let builder = SwapKeysBuilder {
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

    let ix = get_swap_ix(
        &builder,
        IxArgs {
            amount,
            limit,
            inp_lst_index: JUPSOL_FIXTURE_LST_IDX as u32,
            out_lst_index: MSOL_FIXTURE_LST_IDX as u32,
            inp_lst_value_calc_accs: jupsol_fixtures_svc_suf().suf_len() + 1,
            out_lst_value_calc_accs: msol_fixtures_svc_suf().suf_len() + 1,
        },
        ix_type,
    );

    (ix_prefix, ix, builder, inp_calc, out_calc, pricing)
}

pub fn swap_ix_fixtures_accounts_opt(builder: &SwapKeysBuilder) -> Vec<PkAccountTup> {
    fixtures_accounts_opt_cloned(swap_exact_in_ix_keys_owned(builder).seq().copied()).collect()
}

pub fn assert_correct_swap<T: SolValCalc, O: SolValCalc, P: PriceExactIn + PriceExactOut>(
    ix_type: SwapIxType,
    bef: &[PkAccountTup],
    aft: &[PkAccountTup],
    amount: u64,
    inp_mint: &[u8; 32],
    out_mint: &[u8; 32],
    inp_lst_acc: [u8; 32],
    out_lst_acc: [u8; 32],
    pf_accum_acc: [u8; 32],
    out_pool_reserves_acc: [u8; 32],
    inp_calc: T,
    out_calc: O,
    pricing: P,
) -> i128 {
    let [pools, lst_state_lists, inp_lst_accs, out_lst_accs, protocol_fee_accumulator_accs, out_pool_reserves_accs] =
        [
            POOL_STATE_ID,
            LST_STATE_LIST_ID,
            inp_lst_acc,
            out_lst_acc,
            pf_accum_acc,
            out_pool_reserves_acc,
        ]
        .map(|a| acc_bef_aft(&Pubkey::new_from_array(a), bef, aft));

    let [pool_bef, pool_aft] = pools.each_ref().map(|a| {
        PoolStatePacked::of_acc_data(&a.data)
            .unwrap()
            .into_pool_state()
    });

    let [lst_state_list_bef, lst_state_list_aft] = lst_state_lists
        .each_ref()
        .map(|a| LstStatePackedList::of_acc_data(&a.data).unwrap());

    let [[inp_lst_acc_bef, inp_lst_acc_aft], [out_lst_acc_bef, out_lst_acc_aft], [protocol_fee_accumulator_bef, protocol_fee_accumulator_aft], [out_pool_reserves_bef, _]] =
        [
            inp_lst_accs,
            out_lst_accs,
            protocol_fee_accumulator_accs,
            out_pool_reserves_accs,
        ]
        .map(|accs| {
            accs.each_ref()
                .map(|a| RawTokenAccount::of_acc_data(&a.data).unwrap())
        });

    let quote = match ix_type {
        SwapIxType::ExactIn => quote_exact_in(SwapQuoteArgs {
            amt: amount,
            out_reserves: u64::from_le_bytes(out_pool_reserves_bef.amount),
            trading_protocol_fee_bps: pool_bef.trading_protocol_fee_bps,
            inp_mint: *inp_mint,
            out_mint: *out_mint,
            inp_calc,
            out_calc,
            pricing,
        })
        .unwrap(),
        SwapIxType::ExactOut => quote_exact_out(SwapQuoteArgs {
            amt: amount,
            out_reserves: u64::from_le_bytes(out_pool_reserves_bef.amount),
            trading_protocol_fee_bps: pool_bef.trading_protocol_fee_bps,
            inp_mint: *inp_mint,
            out_mint: *out_mint,
            inp_calc,
            out_calc,
            pricing,
        })
        .unwrap(),
    };

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

    pool_sol_val_delta
}
