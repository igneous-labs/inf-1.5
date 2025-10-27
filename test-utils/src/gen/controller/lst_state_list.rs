use std::{
    borrow::Borrow,
    collections::{HashMap, HashSet},
    ops::RangeInclusive,
};

use generic_array_struct::generic_array_struct;
use inf1_ctl_core::{
    accounts::lst_state_list::{LstStatePackedList, LstStatePackedListMut},
    typedefs::lst_state::{LstState, LstStatePacked},
};
use inf1_svc_lido_core::solido_legacy_core::TOKENKEG_PROGRAM;
use jiminy_sysvar_rent::Rent;
use proptest::{collection::vec, prelude::*};
use solana_account::Account;
use solana_pubkey::Pubkey;

use crate::{
    bool_strat, bool_to_u8, create_pool_reserves_ata, create_protocol_fee_accumulator_ata,
    find_pool_reserves_ata, find_protocol_fee_accumulator_ata, gas_diff_zip_assert,
    opt_transpose_strat, pk_strat, u64_strat, u8_to_bool, Diff, WSOL_MINT,
};

#[generic_array_struct(builder pub)]
#[derive(Debug, Clone, Copy, Default)]
pub struct LstStatePks<T> {
    pub mint: T,
    pub sol_value_calculator: T,
}

#[generic_array_struct(builder pub)]
#[derive(Debug, Clone, Copy, Default)]
pub struct LstStateBumps<T> {
    pub pool_reserves_bump: T,
    pub protocol_fee_accumulator_bump: T,
}

#[derive(Debug, Clone, Copy)]
pub struct LstStateData {
    pub lst_state: LstState,
    pub protocol_fee_accumulator: [u8; 32],
    pub pool_reserves: [u8; 32],
}

#[derive(Debug, Clone, Copy, Default)]
pub struct LstStateArgs<I, S, P, U> {
    pub is_input_disabled: I,
    pub sol_value: S,
    pub pks: P,
    pub bumps: U,
}

pub type GenLstStateArgs = LstStateArgs<bool, u64, LstStatePks<[u8; 32]>, LstStateBumps<u8>>;

pub fn gen_lst_state(
    GenLstStateArgs {
        is_input_disabled,
        sol_value,
        pks,
        bumps,
    }: GenLstStateArgs,
    token_prog: &[u8; 32],
) -> LstStateData {
    let protocol_fee_accumulator = create_protocol_fee_accumulator_ata(
        token_prog,
        pks.mint(),
        *bumps.protocol_fee_accumulator_bump(),
    );
    let pool_reserves =
        create_pool_reserves_ata(token_prog, pks.mint(), *bumps.pool_reserves_bump());
    LstStateData {
        lst_state: LstState {
            is_input_disabled: bool_to_u8(is_input_disabled),
            pool_reserves_bump: *bumps.pool_reserves_bump(),
            protocol_fee_accumulator_bump: *bumps.protocol_fee_accumulator_bump(),
            padding: [0u8; 5],
            sol_value,
            mint: *pks.mint(),
            sol_value_calculator: *pks.sol_value_calculator(),
        },
        protocol_fee_accumulator: protocol_fee_accumulator.to_bytes(),
        pool_reserves: pool_reserves.to_bytes(),
    }
}

/// If `Option::None`, `any()` is used. Exceptions:
/// - bumps uses the correct seed derived from find PDA
pub type AnyLstStateArgs = LstStateArgs<
    Option<BoxedStrategy<bool>>,
    Option<BoxedStrategy<u64>>,
    LstStatePks<Option<BoxedStrategy<[u8; 32]>>>,
    LstStateBumps<Option<BoxedStrategy<u8>>>,
>;

pub fn any_lst_state(
    AnyLstStateArgs {
        is_input_disabled,
        sol_value,
        pks,
        bumps,
    }: AnyLstStateArgs,
    token_prog: Option<BoxedStrategy<[u8; 32]>>,
) -> impl Strategy<Value = LstStateData> {
    let is_input_disabled = bool_strat(is_input_disabled);
    let sol_value = u64_strat(sol_value);
    let pks = pks.0.map(pk_strat);
    let token_prog = token_prog.unwrap_or_else(|| Just(TOKENKEG_PROGRAM).boxed());
    let bumps = bumps.0.map(opt_transpose_strat);

    (is_input_disabled, sol_value, pks, token_prog, bumps)
        .prop_map(|(is_input_disabled, sol_value, pks, token_prog, bumps)| {
            let mint = *LstStatePks(pks).mint();
            let bumps = LstStateBumps(bumps);
            let [r_bump, pfa_bump] = [
                (
                    *bumps.pool_reserves_bump(),
                    find_pool_reserves_ata as fn(&[u8; 32], &[u8; 32]) -> (Pubkey, u8),
                ),
                (
                    *bumps.protocol_fee_accumulator_bump(),
                    find_protocol_fee_accumulator_ata,
                ),
            ]
            .map(|(opt, find)| opt.unwrap_or_else(|| find(&token_prog, &mint).1));
            (
                is_input_disabled,
                sol_value,
                pks,
                token_prog,
                r_bump,
                pfa_bump,
            )
        })
        .prop_map(
            |(is_input_disabled, sol_value, pks, token_prog, r_bump, pfa_bump)| {
                gen_lst_state(
                    GenLstStateArgs {
                        is_input_disabled,
                        sol_value,
                        pks: LstStatePks(pks),
                        bumps: NewLstStateBumpsBuilder::start()
                            .with_pool_reserves_bump(r_bump)
                            .with_protocol_fee_accumulator_bump(pfa_bump)
                            .build(),
                    },
                    &token_prog,
                )
            },
        )
}

/// `args.pks` is ignored
pub fn any_wsol_lst_state(args: AnyLstStateArgs) -> impl Strategy<Value = LstStateData> {
    any_lst_state(
        AnyLstStateArgs {
            pks: LstStatePks(
                NewLstStatePksBuilder::start()
                    .with_mint(WSOL_MINT.to_bytes())
                    .with_sol_value_calculator(inf1_svc_wsol_core::ID)
                    .build()
                    .0
                    .map(|x| Some(Just(x).boxed())),
            ),
            ..args
        },
        None,
    )
}

#[derive(Debug, Clone)]
pub struct LstStateListData {
    pub lst_state_list: Vec<u8>,

    /// Map `mint: token acc`
    pub protocol_fee_accumulators: HashMap<[u8; 32], [u8; 32]>,

    /// Map `mint: token acc`
    pub all_pool_reserves: HashMap<[u8; 32], [u8; 32]>,
}

pub fn any_lst_state_list(
    args: AnyLstStateArgs,
    token_prog: Option<BoxedStrategy<[u8; 32]>>,
    len: RangeInclusive<usize>,
) -> impl Strategy<Value = LstStateListData> {
    vec(any_lst_state(args, token_prog), len).prop_map(|v| {
        let mut acc_data = Vec::new();
        let mut protocol_fee_accumulators = HashMap::new();
        let mut all_pool_reserves = HashMap::new();
        let mut dedup_mints = HashSet::new();

        v.into_iter().for_each(
            |LstStateData {
                 lst_state,
                 protocol_fee_accumulator,
                 pool_reserves,
             }| {
                if !dedup_mints.insert(lst_state.mint) {
                    return;
                }
                acc_data.extend(lst_state.as_acc_data_arr());
                protocol_fee_accumulators.insert(lst_state.mint, protocol_fee_accumulator);
                all_pool_reserves.insert(lst_state.mint, pool_reserves);
            },
        );

        LstStateListData {
            lst_state_list: acc_data,
            protocol_fee_accumulators,
            all_pool_reserves,
        }
    })
}

impl LstStateListData {
    /// Returns index that lst state is at in the new lst_state_list
    pub fn upsert(
        &mut self,
        LstStateData {
            lst_state,
            protocol_fee_accumulator,
            pool_reserves,
        }: LstStateData,
    ) -> usize {
        match LstStatePackedListMut::of_acc_data(&mut self.lst_state_list)
            .unwrap()
            .0
            .iter_mut()
            .enumerate()
            .find(|(_, s)| s.into_lst_state().mint == lst_state.mint)
        {
            Some((i, existing)) => {
                *existing = *LstStatePacked::of_acc_data_arr(lst_state.as_acc_data_arr());
                i
            }
            None => {
                self.lst_state_list.extend(lst_state.as_acc_data_arr());
                self.all_pool_reserves.insert(lst_state.mint, pool_reserves);
                self.protocol_fee_accumulators
                    .insert(lst_state.mint, protocol_fee_accumulator);
                LstStatePackedList::of_acc_data(&self.lst_state_list)
                    .unwrap()
                    .0
                    .len()
                    - 1
            }
        }
    }
}

pub fn lst_state_list_account(data: Vec<u8>) -> Account {
    Account {
        lamports: Rent::DEFAULT.min_balance(data.len()),
        data,
        owner: Pubkey::new_from_array(inf1_ctl_core::ID),
        executable: false,
        rent_epoch: u64::MAX,
    }
}

fn lst_state_to_gen_args(
    LstState {
        is_input_disabled,
        pool_reserves_bump,
        protocol_fee_accumulator_bump,
        sol_value,
        mint,
        sol_value_calculator,
        padding: _,
    }: &LstState,
) -> GenLstStateArgs {
    GenLstStateArgs {
        is_input_disabled: u8_to_bool(*is_input_disabled),
        sol_value: *sol_value,
        pks: NewLstStatePksBuilder::start()
            .with_mint(*mint)
            .with_sol_value_calculator(*sol_value_calculator)
            .build(),
        bumps: NewLstStateBumpsBuilder::start()
            .with_pool_reserves_bump(*pool_reserves_bump)
            .with_protocol_fee_accumulator_bump(*protocol_fee_accumulator_bump)
            .build(),
    }
}

pub type DiffLstStateArgs =
    LstStateArgs<Diff<bool>, Diff<u64>, LstStatePks<Diff<[u8; 32]>>, LstStateBumps<Diff<u8>>>;

pub fn assert_diffs_lst_state(
    DiffLstStateArgs {
        is_input_disabled,
        sol_value,
        pks,
        bumps,
    }: &DiffLstStateArgs,
    bef: &LstState,
    aft: &LstState,
) {
    let [GenLstStateArgs {
        is_input_disabled: bef_is_input_disabled,
        sol_value: bef_sol_value,
        pks: bef_pks,
        bumps: bef_bumps,
    }, GenLstStateArgs {
        is_input_disabled: aft_is_input_disabled,
        sol_value: aft_sol_value,
        pks: aft_pks,
        bumps: aft_bumps,
    }] = [bef, aft].map(lst_state_to_gen_args);
    is_input_disabled.assert(&bef_is_input_disabled, &aft_is_input_disabled);
    sol_value.assert(&bef_sol_value, &aft_sol_value);
    gas_diff_zip_assert!(pks, bef_pks, aft_pks);
    gas_diff_zip_assert!(bumps, bef_bumps, aft_bumps);
}

#[derive(Debug, Clone, Copy)]
pub enum LstStateChange {
    Diff(DiffLstStateArgs),
    Add(LstState),
    Del,
}

pub fn assert_diffs_lst_state_list(
    changes: impl IntoIterator<Item = impl Borrow<LstStateChange>>,
    bef: impl IntoIterator<Item = impl Borrow<LstState>>,
    aft: impl IntoIterator<Item = impl Borrow<LstState>>,
) {
    let changes = changes.into_iter();
    let mut bef = bef.into_iter();
    let mut aft = aft.into_iter();
    changes.for_each(|change| match change.borrow() {
        LstStateChange::Diff(d) => {
            assert_diffs_lst_state(
                d,
                bef.next().unwrap().borrow(),
                aft.next().unwrap().borrow(),
            );
        }
        LstStateChange::Add(s) => {
            assert_eq!(s, aft.next().unwrap().borrow());
        }
        LstStateChange::Del => {
            bef.next().unwrap();
        }
    });
    if bef.next().is_some() {
        panic!("bef not exhausted, probably missing deletion");
    }
    if aft.next().is_some() {
        panic!("aft not exhausted, probably missing addition");
    }
}

#[derive(Debug)]
pub struct LstStateListChanges<'a> {
    list: &'a [LstState],
    changes: Vec<LstStateChange>,
}

impl<'a> LstStateListChanges<'a> {
    /// Default is `Diff::NoChange` for all fields
    pub fn new(list: &'a [LstState]) -> Self {
        Self {
            list,
            changes: list
                .iter()
                .map(|_| LstStateChange::Diff(DiffLstStateArgs::default()))
                .collect(),
        }
    }
}

impl LstStateListChanges<'_> {
    pub fn with_push(self, lst_state: LstState) -> Self {
        let Self { list, mut changes } = self;
        changes.push(LstStateChange::Add(lst_state));
        Self { list, changes }
    }

    pub fn with_del_by_mint(self, mint: &[u8; 32]) -> Self {
        let Self { list, mut changes } = self;
        let i = list.iter().position(|l| l.mint == *mint).unwrap();
        changes[i] = LstStateChange::Del;
        Self { list, changes }
    }

    pub fn with_diff_by_mint(self, mint: &[u8; 32], diff: DiffLstStateArgs) -> Self {
        let Self { list, mut changes } = self;
        let i = list.iter().position(|l| l.mint == *mint).unwrap();
        changes[i] = LstStateChange::Diff(diff);
        Self { list, changes }
    }

    pub fn build(self) -> Vec<LstStateChange> {
        let Self { list: _, changes } = self;
        changes
    }
}
