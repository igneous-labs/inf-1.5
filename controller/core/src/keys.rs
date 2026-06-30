use generic_array_struct::generic_array_struct;

use crate::{
    internal_utils::const_map,
    pda::{
        const_find_disable_pool_authority_list, const_find_lst_state_list, const_find_pool_state,
        const_find_protocol_fee, const_find_rebalance_record,
    },
};

#[generic_array_struct(all pub)]
pub struct ConstAccs<T> {
    /// This program's (INF controller) program ID
    pub program: T,

    pub sys_prog: T,
    pub atoken: T,
    pub tokenkeg: T,
    pub token_2022: T,
    pub instructions_sysvar: T,

    // whitelisted SOL value calculator programs
    pub sanctum_spl_svc: T,
    pub sanctum_spl_multi_svc: T,
    pub spl_svc: T,
    pub lido_svc: T,
    pub marinade_svc: T,
    pub wsol_svc: T,
}

pub const CONST_KEY_STRS: ConstAccs<&'static str> = ConstAccs::const_from_destr(ConstAccsDestr {
    program: "5ocnV1qiCgaQR8Jb8xWnVbApfaygJ8tNoZfgPwsgx9kx",
    sys_prog: "11111111111111111111111111111111",
    atoken: "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL",
    tokenkeg: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
    token_2022: "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb",
    instructions_sysvar: "Sysvar1nstructions1111111111111111111111111",
    sanctum_spl_svc: "sspUE1vrh7xRoXxGsg7vR1zde2WdGtJRbyK9uRumBDy",
    sanctum_spl_multi_svc: "ssmbu3KZxgonUtjEMCKspZzxvUQCxAFnyh1rcHUeEDo",
    spl_svc: "sp1V4h2gWorkGhVcazBc22Hfo2f5sd7jcjT4EDPrWFF",
    lido_svc: "1idUSy4MGGKyKhvjSnGZ6Zc7Q4eKQcibym4BkEEw9KR",
    marinade_svc: "mare3SCyfZkAndpBRBeonETmkCCB3TJTTrz8ZN2dnhP",
    wsol_svc: "wsoGmxQLSvwWpuaidCApxN5kEowLe2HLQLJhCQnj4bE",
});

pub const CONST_KEYS_OWNED: ConstAccs<[u8; 32]> = ConstAccs(const_map!(
    [0; 32],
    CONST_KEY_STRS.0,
    const_crypto::bs58::decode_pubkey
));

// Convenience re-exports from ConstAccs for backward compatibility.
// New code should prefer `CONST_KEY_STRS.field()` / `*CONST_KEYS_OWNED.field()` directly.

#[deprecated = "Use `CONST_KEY_STRS.sys_prog()` instead"]
pub const SYS_PROG_ID_STR: &str = CONST_KEY_STRS.sys_prog();
#[deprecated = "Use `*CONST_KEYS_OWNED.sys_prog()` instead"]
pub const SYS_PROG_ID: [u8; 32] = *CONST_KEYS_OWNED.sys_prog();

#[deprecated = "Use `CONST_KEY_STRS.atoken()` instead"]
pub const ATOKEN_ID_STR: &str = CONST_KEY_STRS.atoken();
#[deprecated = "Use `*CONST_KEYS_OWNED.atoken()` instead"]
pub const ATOKEN_ID: [u8; 32] = *CONST_KEYS_OWNED.atoken();

#[deprecated = "Use `CONST_KEY_STRS.tokenkeg()` instead"]
pub const TOKENKEG_ID_STR: &str = CONST_KEY_STRS.tokenkeg();
#[deprecated = "Use `*CONST_KEYS_OWNED.tokenkeg()` instead"]
pub const TOKENKEG_ID: [u8; 32] = *CONST_KEYS_OWNED.tokenkeg();

#[deprecated = "Use `CONST_KEY_STRS.token_2022()` instead"]
pub const TOKEN_2022_ID_STR: &str = CONST_KEY_STRS.token_2022();
#[deprecated = "Use `*CONST_KEYS_OWNED.token_2022()` instead"]
pub const TOKEN_2022_ID: [u8; 32] = *CONST_KEYS_OWNED.token_2022();

#[deprecated = "Use `CONST_KEY_STRS.instructions_sysvar()` instead"]
pub const INSTRUCTIONS_SYSVAR_ID_STR: &str = CONST_KEY_STRS.instructions_sysvar();
#[deprecated = "Use `*CONST_KEYS_OWNED.instructions_sysvar()` instead"]
pub const INSTRUCTIONS_SYSVAR_ID: [u8; 32] = *CONST_KEYS_OWNED.instructions_sysvar();

#[deprecated = "Use `CONST_KEY_STRS.sanctum_spl_svc()` instead"]
pub const SANCTUM_SPL_SVC_ID_STR: &str = CONST_KEY_STRS.sanctum_spl_svc();
#[deprecated = "Use `*CONST_KEYS_OWNED.sanctum_spl_svc()` instead"]
pub const SANCTUM_SPL_SVC_ID: [u8; 32] = *CONST_KEYS_OWNED.sanctum_spl_svc();

#[deprecated = "Use `CONST_KEY_STRS.sanctum_spl_multi_svc()` instead"]
pub const SANCTUM_SPL_MULTI_SVC_ID_STR: &str = CONST_KEY_STRS.sanctum_spl_multi_svc();
#[deprecated = "Use `*CONST_KEYS_OWNED.sanctum_spl_multi_svc()` instead"]
pub const SANCTUM_SPL_MULTI_SVC_ID: [u8; 32] = *CONST_KEYS_OWNED.sanctum_spl_multi_svc();

#[deprecated = "Use `CONST_KEY_STRS.spl_svc()` instead"]
pub const SPL_SVC_ID_STR: &str = CONST_KEY_STRS.spl_svc();
#[deprecated = "Use `*CONST_KEYS_OWNED.spl_svc()` instead"]
pub const SPL_SVC_ID: [u8; 32] = *CONST_KEYS_OWNED.spl_svc();

#[deprecated = "Use `CONST_KEY_STRS.lido_svc()` instead"]
pub const LIDO_SVC_ID_STR: &str = CONST_KEY_STRS.lido_svc();
#[deprecated = "Use `*CONST_KEYS_OWNED.lido_svc()` instead"]
pub const LIDO_SVC_ID: [u8; 32] = *CONST_KEYS_OWNED.lido_svc();

#[deprecated = "Use `CONST_KEY_STRS.marinade_svc()` instead"]
pub const MARINADE_SVC_ID_STR: &str = CONST_KEY_STRS.marinade_svc();
#[deprecated = "Use `*CONST_KEYS_OWNED.marinade_svc()` instead"]
pub const MARINADE_SVC_ID: [u8; 32] = *CONST_KEYS_OWNED.marinade_svc();

#[deprecated = "Use `CONST_KEY_STRS.wsol_svc()` instead"]
pub const WSOL_SVC_ID_STR: &str = CONST_KEY_STRS.wsol_svc();
#[deprecated = "Use `*CONST_KEYS_OWNED.wsol_svc()` instead"]
pub const WSOL_SVC_ID: [u8; 32] = *CONST_KEYS_OWNED.wsol_svc();

macro_rules! const_pda {
    ($INTER:ident, $ID_STR:ident, $ID:ident, $BUMP:ident, $const_find:expr) => {
        const $INTER: ([u8; 32], u8) = $const_find(CONST_KEYS_OWNED.program());
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

// Hardcoded whitelisted sol value calculator program IDs.
// Duplicated with consts in the other svc crates,
// but declaring them separately here to avoid adding another dependency

pub const WHITELISTED_SVC_PROGS: [[u8; 32]; 6] = [
    *CONST_KEYS_OWNED.sanctum_spl_svc(),
    *CONST_KEYS_OWNED.sanctum_spl_multi_svc(),
    *CONST_KEYS_OWNED.spl_svc(),
    *CONST_KEYS_OWNED.lido_svc(),
    *CONST_KEYS_OWNED.marinade_svc(),
    *CONST_KEYS_OWNED.wsol_svc(),
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
