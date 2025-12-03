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

id_str!(
    INF_MINT_ID_STR,
    INF_MINT_ID,
    "5oVNBeEEQvYi1cX3ir8Dx5n1P7pdxydbGF2X4TxVusJm"
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
}
