use crate::pda::{
    const_find_disable_pool_authority_list, const_find_lst_state_list, const_find_pool_state,
    const_find_protocol_fee, const_find_rebalance_record,
};

macro_rules! id_str {
    ($ID_STR:ident, $ID:ident, $pkstr:expr) => {
        pub const $ID_STR: &str = $pkstr;
        pub const $ID: [u8; 32] = const_crypto::bs58::decode_pubkey($ID_STR);
    };
}
pub(crate) use id_str;

id_str!(
    SYS_PROG_ID_STR,
    SYS_PROG_ID,
    "11111111111111111111111111111111"
);

id_str!(
    ATOKEN_ID_STR,
    ATOKEN_ID,
    "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"
);

id_str!(
    TOKENKEG_ID_STR,
    TOKENKEG_ID,
    "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
);

id_str!(
    TOKEN_2022_ID_STR,
    TOKEN_2022_ID,
    "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb"
);

id_str!(
    INSTRUCTIONS_SYSVAR_ID_STR,
    INSTRUCTIONS_SYSVAR_ID,
    "Sysvar1nstructions1111111111111111111111111"
);

macro_rules! const_pda {
    ($INTER:ident, $ID_STR:ident, $ID:ident, $BUMP:ident, $const_find:expr) => {
        const $INTER: ([u8; 32], u8) = $const_find(&crate::ID);
        pub const $ID: [u8; 32] = $INTER.0;
        pub const $BUMP: u8 = $INTER.1;
        pub const $ID_STR: &str = const_crypto::bs58::encode_pubkey(&$ID).str();
    };
}

const_pda!(
    POOL_STATE,
    POOL_STATE_ID_STR,
    POOL_STATE_ID,
    POOL_STATE_BUMP,
    const_find_pool_state
);

const_pda!(
    LST_STATE_LIST,
    LST_STATE_LIST_ID_STR,
    LST_STATE_LIST_ID,
    LST_STATE_LIST_BUMP,
    const_find_lst_state_list
);

const_pda!(
    PROTOCOL_FEE,
    PROTOCOL_FEE_ID_STR,
    PROTOCOL_FEE_ID,
    PROTOCOL_FEE_BUMP,
    const_find_protocol_fee
);

const_pda!(
    REBALANCE_RECORD,
    REBALANCE_RECORD_ID_STR,
    REBALANCE_RECORD_ID,
    REBALANCE_RECORD_BUMP,
    const_find_rebalance_record
);

const_pda!(
    DISABLE_POOL_AUTHORITY_LIST,
    DISABLE_POOL_AUTHORITY_LIST_ID_STR,
    DISABLE_POOL_AUTHORITY_LIST_ID,
    DISABLE_POOL_AUTHORITY_LIST_BUMP,
    const_find_disable_pool_authority_list
);

// Hardcoded whitelisted sol value calculator program IDs
// Duplicated with consts in the other svc crates,
// but separating them here to adding another dependency

id_str!(
    SANCTUM_SPL_SVC_ID_STR,
    SANCTUM_SPL_SVC_ID,
    "sspUE1vrh7xRoXxGsg7vR1zde2WdGtJRbyK9uRumBDy"
);
id_str!(
    SANCTUM_SPL_MULTI_SVC_ID_STR,
    SANCTUM_SPL_MULTI_SVC_ID,
    "ssmbu3KZxgonUtjEMCKspZzxvUQCxAFnyh1rcHUeEDo"
);
id_str!(
    SPL_SVC_ID_STR,
    SPL_SVC_ID,
    "sp1V4h2gWorkGhVcazBc22Hfo2f5sd7jcjT4EDPrWFF"
);
id_str!(
    LIDO_SVC_ID_STR,
    LIDO_SVC_ID,
    "1idUSy4MGGKyKhvjSnGZ6Zc7Q4eKQcibym4BkEEw9KR"
);
id_str!(
    MARINADE_SVC_ID_STR,
    MARINADE_SVC_ID,
    "mare3SCyfZkAndpBRBeonETmkCCB3TJTTrz8ZN2dnhP"
);
id_str!(
    WSOL_SVC_ID_STR,
    WSOL_SVC_ID,
    "wsoGmxQLSvwWpuaidCApxN5kEowLe2HLQLJhCQnj4bE"
);

pub const WHITELISTED_SVC_PROGS: [[u8; 32]; 6] = [
    SANCTUM_SPL_SVC_ID,
    SANCTUM_SPL_MULTI_SVC_ID,
    SPL_SVC_ID,
    LIDO_SVC_ID,
    MARINADE_SVC_ID,
    WSOL_SVC_ID,
];

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use super::*;

    #[test]
    fn const_pda_snapshots() {
        [
            (
                expect!["AYhux5gJzCoeoc1PoJ1VxwPDe22RwcvpHviLDD1oCGvW"],
                POOL_STATE_ID_STR,
            ),
            (
                expect!["Gb7m4daakbVbrFLR33FKMDVMHAprRZ66CSYt4bpFwUgS"],
                LST_STATE_LIST_ID_STR,
            ),
            (
                expect!["6U8Ve7NuTVq9pb3xEC2ZwxBhceWULUuJn1nSKCTraq5r"],
                PROTOCOL_FEE_ID_STR,
            ),
            (
                expect!["GVoB1QdoqCzdSsQr7zsxyGZB1HhWpfejm6ZZduvseSNa"],
                REBALANCE_RECORD_ID_STR,
            ),
            (
                expect!["FJc6b3iyYaD5p24aKQ2FcM7WVATapPGq65LhY1MDKXzG"],
                DISABLE_POOL_AUTHORITY_LIST_ID_STR,
            ),
        ]
        .into_iter()
        .for_each(|(e, s)| e.assert_eq(s));
    }

    #[test]
    fn whitelisted_svcs_snapshot() {
        let all: String = WHITELISTED_SVC_PROGS
            .iter()
            .flat_map(|pk| {
                [
                    const_crypto::bs58::encode_pubkey(pk).str().to_owned(),
                    ",\n".to_owned(),
                ]
            })
            .collect();
        expect![[r#"
            sspUE1vrh7xRoXxGsg7vR1zde2WdGtJRbyK9uRumBDy,
            ssmbu3KZxgonUtjEMCKspZzxvUQCxAFnyh1rcHUeEDo,
            sp1V4h2gWorkGhVcazBc22Hfo2f5sd7jcjT4EDPrWFF,
            1idUSy4MGGKyKhvjSnGZ6Zc7Q4eKQcibym4BkEEw9KR,
            mare3SCyfZkAndpBRBeonETmkCCB3TJTTrz8ZN2dnhP,
            wsoGmxQLSvwWpuaidCApxN5kEowLe2HLQLJhCQnj4bE,
        "#]]
        .assert_eq(&all);
    }
}
