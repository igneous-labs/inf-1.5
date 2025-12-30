#![allow(deprecated)]

use std::collections::HashMap;

use inf1_pp_ag_std::update::all::Pair;
use inf1_std::{
    err::InfErr,
    inf1_ctl_core::{accounts::pool_state::PoolState, typedefs::lst_state::LstState},
    quote::{liquidity::add::AddLiqQuoteErr, swap::err::SwapQuoteErr},
    InfStd,
};
use inf1_svc_ag_std::{
    inf1_svc_lido_std::{solido_legacy_core::STSOL_MINT_ADDR, LidoSvcStd},
    inf1_svc_wsol_std::WsolSvcStd,
    SvcAg, SvcAgStd,
};
use inf1_test_utils::{bool_to_u8, WSOL_MINT};

use crate::common::{create_pda, find_pda, lst_state_list_fixture, pool_state_fixture};

const DISABLED_MINT: [u8; 32] = STSOL_MINT_ADDR;
const DUMMY_AMT: u64 = 1_000_000_000;
const DISABLED_INP_PAIR: Pair<&[u8; 32]> = Pair {
    inp: &DISABLED_MINT,
    out: WSOL_MINT.as_array(),
};

fn disable_lst_input(list: &mut [LstState], disable: &[u8; 32]) {
    list.iter_mut()
        .find(|s| s.mint == *disable)
        .unwrap()
        .is_input_disabled = bool_to_u8(true)
}

fn svcs_for_test() -> HashMap<[u8; 32], SvcAgStd> {
    [
        (
            DISABLED_MINT,
            SvcAgStd(SvcAg::Lido(LidoSvcStd {
                calc: Some(Default::default()),
            })),
        ),
        (WSOL_MINT.to_bytes(), SvcAgStd(SvcAg::Wsol(WsolSvcStd))),
    ]
    .into_iter()
    .collect()
}

fn inf_for_test(pool: PoolState, list: &[LstState]) -> InfStd {
    InfStd::new(
        pool,
        list.iter().flat_map(|s| *s.as_acc_data_arr()).collect(),
        Some(1_000_000_000),
        None,
        Default::default(),
        svcs_for_test(),
        Default::default(),
        find_pda,
        create_pda,
    )
    .unwrap()
}

fn inp_disabled_setup() -> InfStd {
    let pool = pool_state_fixture();
    let mut list = lst_state_list_fixture();
    disable_lst_input(&mut list, &DISABLED_MINT);
    inf_for_test(pool, &list)
}

#[test]
fn quote_add_liq_inp_disabled_fixture() {
    let mut inf = inp_disabled_setup();

    let e = inf.quote_add_liq(&DISABLED_MINT, DUMMY_AMT).unwrap_err();
    let em = inf
        .quote_add_liq_mut(&DISABLED_MINT, DUMMY_AMT)
        .unwrap_err();
    assert_eq!(e, InfErr::AddLiqQuote(AddLiqQuoteErr::InpDisabled));
    assert_eq!(em, InfErr::AddLiqQuote(AddLiqQuoteErr::InpDisabled));
}

#[test]
fn quote_exact_in_inp_disabled_fixture() {
    let mut inf = inp_disabled_setup();

    let e = inf
        .quote_exact_in(&DISABLED_INP_PAIR, DUMMY_AMT)
        .unwrap_err();
    let em = inf
        .quote_exact_in_mut(&DISABLED_INP_PAIR, DUMMY_AMT)
        .unwrap_err();
    assert_eq!(e, InfErr::SwapQuote(SwapQuoteErr::InpDisabled));
    assert_eq!(em, InfErr::SwapQuote(SwapQuoteErr::InpDisabled));
}

#[test]
fn quote_exact_out_inp_disabled_fixture() {
    let mut inf = inp_disabled_setup();

    let e = inf
        .quote_exact_out(&DISABLED_INP_PAIR, DUMMY_AMT)
        .unwrap_err();
    let em = inf
        .quote_exact_out_mut(&DISABLED_INP_PAIR, DUMMY_AMT)
        .unwrap_err();
    assert_eq!(e, InfErr::SwapQuote(SwapQuoteErr::InpDisabled));
    assert_eq!(em, InfErr::SwapQuote(SwapQuoteErr::InpDisabled));
}
