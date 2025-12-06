use core::mem::take;

use generic_array_struct::generic_array_struct;
use inf1_core::instructions::swap::IxAccs;
use inf1_ctl_jiminy::{
    instructions::swap::v2::{
        exact_in::NewSwapExactInV2IxPreAccsBuilder, IxPreKeysOwned, NewSwapEntryAccsBuilder,
        SwapEntryAccs,
    },
    keys::{LST_STATE_LIST_ID, POOL_STATE_ID},
};
use inf1_pp_core::pair::Pair;
use inf1_test_utils::{
    fill_mock_prog_accs, lst_state_list_account, mock_mint, mock_sys_acc, mock_token_acc, raw_mint,
    raw_token_acc, AccountMap, LstStateListData, VerPoolState,
};
use solana_account::Account;
use solana_pubkey::Pubkey;

pub fn fill_swap_prog_accs<I, C, D, P>(
    am: &mut AccountMap,
    IxAccs {
        inp_calc_prog,
        out_calc_prog,
        pricing_prog,
        ..
    }: &IxAccs<[u8; 32], I, C, D, P>,
) {
    fill_mock_prog_accs(am, [*inp_calc_prog, *out_calc_prog, *pricing_prog]);
}

#[generic_array_struct(builder pub)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SwapTokenU64s<T> {
    pub reserves_bal: T,
    pub acc_bal: T,
    pub mint_supply: T,
}

#[generic_array_struct(builder pub)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SwapTokenAddrs<T> {
    pub mint: T,
    pub acc: T,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SwapTokenArg<U, A> {
    pub u64s: SwapTokenU64s<U>,
    pub addrs: SwapTokenAddrs<A>,
}

type SwapTokenArgVals = SwapTokenArg<u64, [u8; 32]>;

/// Assumes
/// - both mints are tokenkeg mints
/// - both tokens are 9 decimals
pub fn swap_pre_accs(
    signer: &[u8; 32],
    ps: &VerPoolState,
    lsl: &LstStateListData,
    args: &Pair<SwapTokenArgVals>,
) -> (IxPreKeysOwned, AccountMap) {
    let mut pair = args.map(|args| {
        if args.addrs.mint() == ps.lp_token_mint() {
            lp_accs(ps, signer, &args)
        } else {
            lst_accs(lsl, signer, &args)
        }
    });
    let accounts = NewSwapExactInV2IxPreAccsBuilder::start()
        .with_signer(((*signer).into(), mock_sys_acc(1_000_000_000)))
        .with_pool_state((POOL_STATE_ID.into(), ps.into_account()))
        .with_lst_state_list((
            LST_STATE_LIST_ID.into(),
            lst_state_list_account(lsl.lst_state_list.clone()),
        ))
        // move instead of clone
        .with_inp_mint(take(pair.inp.mint_mut()))
        .with_inp_acc(take(pair.inp.acc_mut()))
        .with_inp_token_program(take(pair.inp.token_program_mut()))
        .with_inp_pool_reserves(take(pair.inp.pool_reserves_mut()))
        .with_out_mint(take(pair.out.mint_mut()))
        .with_out_acc(take(pair.out.acc_mut()))
        .with_out_token_program(take(pair.out.token_program_mut()))
        .with_out_pool_reserves(take(pair.out.pool_reserves_mut()))
        .build();
    let ix_prefix = IxPreKeysOwned::new(accounts.0.each_ref().map(|(pk, _)| pk.to_bytes()));

    (ix_prefix, accounts.0.into_iter().collect())
}

fn lp_accs(
    ps: &VerPoolState,
    signer: &[u8; 32],
    SwapTokenArg { u64s, addrs }: &SwapTokenArgVals,
) -> SwapEntryAccs<(Pubkey, Account)> {
    if u64s.reserves_bal() != u64s.mint_supply() {
        panic!(
            "reserves_bal {} != mint_supply {}. Set both to eq.",
            u64s.reserves_bal(),
            u64s.mint_supply()
        );
    }
    let lp_mint = (
        Pubkey::new_from_array(*ps.lp_token_mint()),
        mock_mint(raw_mint(Some(POOL_STATE_ID), None, *u64s.mint_supply(), 9)),
    );
    let acc = (
        Pubkey::new_from_array(*addrs.acc()),
        mock_token_acc(raw_token_acc(*ps.lp_token_mint(), *signer, *u64s.acc_bal())),
    );
    NewSwapEntryAccsBuilder::start()
        .with_mint(lp_mint.clone())
        .with_acc(acc)
        .with_pool_reserves(lp_mint)
        .with_token_program(mollusk_svm_programs_token::token::keyed_account())
        .build()
}

fn lst_accs(
    lsl: &LstStateListData,
    signer: &[u8; 32],
    SwapTokenArg { u64s, addrs }: &SwapTokenArgVals,
) -> SwapEntryAccs<(Pubkey, Account)> {
    let mint = (
        Pubkey::new_from_array(*addrs.mint()),
        // dont-care abt mint and freeze auths
        mock_mint(raw_mint(None, None, *u64s.mint_supply(), 9)),
    );
    let acc = (
        Pubkey::new_from_array(*addrs.acc()),
        mock_token_acc(raw_token_acc(*addrs.mint(), *signer, *u64s.acc_bal())),
    );
    let reserves = (
        lsl.all_pool_reserves[addrs.mint()].into(),
        mock_token_acc(raw_token_acc(
            *addrs.mint(),
            POOL_STATE_ID,
            *u64s.reserves_bal(),
        )),
    );
    NewSwapEntryAccsBuilder::start()
        .with_mint(mint)
        .with_acc(acc)
        .with_pool_reserves(reserves)
        .with_token_program(mollusk_svm_programs_token::token::keyed_account())
        .build()
}
