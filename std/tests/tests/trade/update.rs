use std::collections::HashMap;

use expect_test::expect;
use inf1_pp_ag_std::update::all::Pair;
use inf1_std::{
    inf1_ctl_core::{keys::CONST_KEYS_OWNED, token_info::TokenInfo, typedefs::lst_state::LstState},
    pda::{find_ata, POOL_STATE_SEED, PROTOCOL_FEE_SEED},
    trade::TradeLimitTy,
    InfStd,
};
use inf1_svc_ag_std::{
    inf1_svc_lido_std::{solido_legacy_core::STSOL_MINT_ADDR, LidoSvcStd},
    inf1_svc_wsol_std::WsolSvcStd,
    SvcAg, SvcAgStd,
};
use inf1_test_utils::WSOL_MINT;
use solana_pubkey::Pubkey;

use crate::common::{create_pda, find_pda, pool_state_fixture};

const OTHER_PROG_ID: Pubkey = Pubkey::from_str_const("un27kVAKYscfzvrkNeYkNZ74tW9o4txuArAweftjakw");

fn svcs_for_test() -> HashMap<[u8; 32], SvcAgStd> {
    [
        (
            STSOL_MINT_ADDR,
            SvcAgStd(SvcAg::Lido(LidoSvcStd {
                calc: Some(Default::default()),
            })),
        ),
        (WSOL_MINT.to_bytes(), SvcAgStd(SvcAg::Wsol(WsolSvcStd))),
    ]
    .into_iter()
    .collect()
}

fn inf_for_test(prog_id: Option<[u8; 32]>) -> InfStd {
    let svcs = svcs_for_test();

    let lsl: Vec<_> = svcs
        .iter()
        .map(|(mint, svc)| {
            let prog_id = prog_id.unwrap_or(*CONST_KEYS_OWNED.program());
            // make sure to use correct bump else create_pda will error
            // due to possible off-curve check
            let [pool_reserves_bump, protocol_fee_accumulator_bump] =
                [POOL_STATE_SEED.as_slice(), &PROTOCOL_FEE_SEED].map(|s| {
                    let auth = find_pda(&[s], &prog_id).unwrap().0;
                    find_ata(find_pda, &auth, &TokenInfo::tokenkeg(mint))
                        .unwrap()
                        .1
                });
            LstState {
                mint: *mint,
                sol_value_calculator: *svc.0.svc_program_id(),
                pool_reserves_bump,
                protocol_fee_accumulator_bump,
                is_input_disabled: Default::default(),
                padding: Default::default(),
                sol_value: Default::default(),
            }
        })
        .collect();

    InfStd::new(
        prog_id,
        pool_state_fixture(),
        lsl.iter().flat_map(|s| *s.as_acc_data_arr()).collect(),
        Some(1_000_000_000),
        None,
        Default::default(),
        svcs,
        Default::default(),
        find_pda,
        create_pda,
    )
    .unwrap()
}

#[test]
fn add_liq_accs_to_update_snapshot() {
    [
        (
            expect![[r#"
        [
            "AYhux5gJzCoeoc1PoJ1VxwPDe22RwcvpHviLDD1oCGvW",
            "Gb7m4daakbVbrFLR33FKMDVMHAprRZ66CSYt4bpFwUgS",
            "F2AETMoKjZgb3965ee9DiSriVmFDMA9Uf1ebuWuVzjUu",
            "AYhux5gJzCoeoc1PoJ1VxwPDe22RwcvpHviLDD1oCGvW",
            "5oVNBeEEQvYi1cX3ir8Dx5n1P7pdxydbGF2X4TxVusJm",
            "SysvarC1ock11111111111111111111111111111111",
            "4T9YzXnmQFMyYi2nrxyXjhtUANavmCkxGCsU3GKaNjwT",
        ]
    "#]],
            None,
        ),
        (
            expect![[r#"
                [
                    "9zMRqtjkTvUm4kVtz2MrPiJnr9spUmYsr8Uqis7y3Brv",
                    "Brg7vhSTVp76eTy3xjwRgBUfh711eH61io2Xvqj72UA5",
                    "6nBpYJ3oeraht4cFyFPj3TLFpNuy8SRMQA2KGRXVAEHY",
                    "9zMRqtjkTvUm4kVtz2MrPiJnr9spUmYsr8Uqis7y3Brv",
                    "5oVNBeEEQvYi1cX3ir8Dx5n1P7pdxydbGF2X4TxVusJm",
                    "SysvarC1ock11111111111111111111111111111111",
                    "4T9YzXnmQFMyYi2nrxyXjhtUANavmCkxGCsU3GKaNjwT",
                ]
            "#]],
            Some(OTHER_PROG_ID),
        ),
    ]
    .into_iter()
    .for_each(|(e, prog_id)| {
        let mut inf = inf_for_test(prog_id.map(|a| a.to_bytes()));

        let out = *inf.pool.lp_token_mint();
        let pair = Pair {
            inp: WSOL_MINT.as_array(),
            out: &out,
        };

        let [immut_in, immut_out] =
            [TradeLimitTy::ExactIn(()), TradeLimitTy::ExactOut(())].map(|lim| {
                inf.accounts_to_update_trade(&pair, lim)
                    .unwrap()
                    .map(|a| Pubkey::new_from_array(a).to_string())
                    .collect::<Vec<_>>()
            });
        let [mut_in, mut_out] =
            [TradeLimitTy::ExactIn(()), TradeLimitTy::ExactOut(())].map(|lim| {
                inf.accounts_to_update_trade_mut(&pair, lim)
                    .unwrap()
                    .map(|a| Pubkey::new_from_array(a).to_string())
                    .collect::<Vec<_>>()
            });

        assert_eq!(immut_in, immut_out);
        assert_eq!(immut_out, mut_in);
        assert_eq!(mut_in, mut_out);

        e.assert_debug_eq(&immut_in);
    });
}
