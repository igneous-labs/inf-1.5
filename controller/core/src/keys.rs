use generic_array_struct::generic_array_struct;

use crate::{
    internal_utils::const_map,
    pda::{CONST_PDA_BUMPS, CONST_PDA_KEYS_OWNED, CONST_PDA_KEY_STRS},
};

#[generic_array_struct(all pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConstAccs<T> {
    /// This program's (INF controller) program ID
    pub program: T,

    /// Hardcoded initial pool admin
    pub init_admin: T,

    /// Default pricing program ID
    pub default_pp: T,

    pub sys_prog: T,
    pub atoken: T,
    pub tokenkeg: T,
    pub token_2022: T,
    pub instructions_sysvar: T,
}

macro_rules! id_str {
    ($ID_STR:ident, $ID:ident, $pkstr:expr) => {
        pub const $ID_STR: &str = $pkstr;
        pub const $ID: [u8; 32] = const_crypto::bs58::decode_pubkey($ID_STR);
    };
}

id_str!(
    INF_PROGRAM_ID_STR,
    INF_PROGRAM_ID,
    "5ocnV1qiCgaQR8Jb8xWnVbApfaygJ8tNoZfgPwsgx9kx"
);

id_str!(
    RESERVE_V2_PROGRAM_ID_STR,
    RESERVE_V2_PROGRAM_ID,
    "un27kVAKYscfzvrkNeYkNZ74tW9o4txuArAweftjakw"
);

pub const PROGRAM_ID_STR: &str = if cfg!(feature = "reserve-v2") {
    RESERVE_V2_PROGRAM_ID_STR
} else {
    INF_PROGRAM_ID_STR
};

pub const CONST_KEY_STRS: ConstAccs<&'static str> = ConstAccs::const_from_destr(ConstAccsDestr {
    program: PROGRAM_ID_STR,
    init_admin: "GRwm4EXMyVwtftQeTft7DZT3HBRxx439PrKq4oM6BwoZ",
    default_pp: "s1b6NRXj6ygNu1QMKXh2H9LUR2aPApAAm1UQ2DjdhNV",
    sys_prog: "11111111111111111111111111111111",
    atoken: "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL",
    tokenkeg: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
    token_2022: "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb",
    instructions_sysvar: "Sysvar1nstructions1111111111111111111111111",
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

#[deprecated = "Use `WHITELISTED_SVC_PROGS_STR.sanctum_spl_svc()` instead"]
pub const SANCTUM_SPL_SVC_ID_STR: &str = WHITELISTED_SVC_PROGS_STR.sanctum_spl_svc();
#[deprecated = "Use `*WHITELISTED_SVC_PROGS.sanctum_spl_svc()` instead"]
pub const SANCTUM_SPL_SVC_ID: [u8; 32] = *WHITELISTED_SVC_PROGS.sanctum_spl_svc();

#[deprecated = "Use `WHITELISTED_SVC_PROGS_STR.sanctum_spl_multi_svc()` instead"]
pub const SANCTUM_SPL_MULTI_SVC_ID_STR: &str = WHITELISTED_SVC_PROGS_STR.sanctum_spl_multi_svc();
#[deprecated = "Use `*WHITELISTED_SVC_PROGS.sanctum_spl_multi_svc()` instead"]
pub const SANCTUM_SPL_MULTI_SVC_ID: [u8; 32] = *WHITELISTED_SVC_PROGS.sanctum_spl_multi_svc();

#[deprecated = "Use `WHITELISTED_SVC_PROGS_STR.spl_svc()` instead"]
pub const SPL_SVC_ID_STR: &str = WHITELISTED_SVC_PROGS_STR.spl_svc();
#[deprecated = "Use `*WHITELISTED_SVC_PROGS.spl_svc()` instead"]
pub const SPL_SVC_ID: [u8; 32] = *WHITELISTED_SVC_PROGS.spl_svc();

#[deprecated = "Use `WHITELISTED_SVC_PROGS_STR.lido_svc()` instead"]
pub const LIDO_SVC_ID_STR: &str = WHITELISTED_SVC_PROGS_STR.lido_svc();
#[deprecated = "Use `*WHITELISTED_SVC_PROGS.lido_svc()` instead"]
pub const LIDO_SVC_ID: [u8; 32] = *WHITELISTED_SVC_PROGS.lido_svc();

#[deprecated = "Use `WHITELISTED_SVC_PROGS_STR.marinade_svc()` instead"]
pub const MARINADE_SVC_ID_STR: &str = WHITELISTED_SVC_PROGS_STR.marinade_svc();
#[deprecated = "Use `*WHITELISTED_SVC_PROGS.marinade_svc()` instead"]
pub const MARINADE_SVC_ID: [u8; 32] = *WHITELISTED_SVC_PROGS.marinade_svc();

#[deprecated = "Use `WHITELISTED_SVC_PROGS_STR.wsol_svc()` instead"]
pub const WSOL_SVC_ID_STR: &str = WHITELISTED_SVC_PROGS_STR.wsol_svc();
#[deprecated = "Use `*WHITELISTED_SVC_PROGS.wsol_svc()` instead"]
pub const WSOL_SVC_ID: [u8; 32] = *WHITELISTED_SVC_PROGS.wsol_svc();

#[deprecated = "Use `CONST_PDA_KEYS_OWNED.pool_state()` instead"]
pub const POOL_STATE_ID: [u8; 32] = *CONST_PDA_KEYS_OWNED.pool_state();
#[deprecated = "Use `CONST_PDA_KEY_STRS.pool_state()` instead"]
pub const POOL_STATE_ID_STR: &str = CONST_PDA_KEY_STRS.pool_state();
#[deprecated = "Use `CONST_PDA_BUMPS.pool_state()` instead"]
pub const POOL_STATE_BUMP: u8 = *CONST_PDA_BUMPS.pool_state();

#[deprecated = "Use `CONST_PDA_KEYS_OWNED.lst_state_list()` instead"]
pub const LST_STATE_LIST_ID: [u8; 32] = *CONST_PDA_KEYS_OWNED.lst_state_list();
#[deprecated = "Use `CONST_PDA_KEY_STRS.lst_state_list()` instead"]
pub const LST_STATE_LIST_ID_STR: &str = CONST_PDA_KEY_STRS.lst_state_list();
#[deprecated = "Use `CONST_PDA_BUMPS.lst_state_list()` instead"]
pub const LST_STATE_LIST_BUMP: u8 = *CONST_PDA_BUMPS.lst_state_list();

#[deprecated = "Use `CONST_PDA_KEYS_OWNED.protocol_fee()` instead"]
pub const PROTOCOL_FEE_ID: [u8; 32] = *CONST_PDA_KEYS_OWNED.protocol_fee();
#[deprecated = "Use `CONST_PDA_KEY_STRS.protocol_fee()` instead"]
pub const PROTOCOL_FEE_ID_STR: &str = CONST_PDA_KEY_STRS.protocol_fee();
#[deprecated = "Use `CONST_PDA_BUMPS.protocol_fee()` instead"]
pub const PROTOCOL_FEE_BUMP: u8 = *CONST_PDA_BUMPS.protocol_fee();

#[deprecated = "Use `CONST_PDA_KEYS_OWNED.rebalance_record()` instead"]
pub const REBALANCE_RECORD_ID: [u8; 32] = *CONST_PDA_KEYS_OWNED.rebalance_record();
#[deprecated = "Use `CONST_PDA_KEY_STRS.rebalance_record()` instead"]
pub const REBALANCE_RECORD_ID_STR: &str = CONST_PDA_KEY_STRS.rebalance_record();
#[deprecated = "Use `CONST_PDA_BUMPS.rebalance_record()` instead"]
pub const REBALANCE_RECORD_BUMP: u8 = *CONST_PDA_BUMPS.rebalance_record();

#[deprecated = "Use `CONST_PDA_KEYS_OWNED.disable_pool_authority_list()` instead"]
pub const DISABLE_POOL_AUTHORITY_LIST_ID: [u8; 32] =
    *CONST_PDA_KEYS_OWNED.disable_pool_authority_list();
#[deprecated = "Use `CONST_PDA_KEY_STRS.disable_pool_authority_list()` instead"]
pub const DISABLE_POOL_AUTHORITY_LIST_ID_STR: &str =
    CONST_PDA_KEY_STRS.disable_pool_authority_list();
#[deprecated = "Use `CONST_PDA_BUMPS.disable_pool_authority_list()` instead"]
pub const DISABLE_POOL_AUTHORITY_LIST_BUMP: u8 = *CONST_PDA_BUMPS.disable_pool_authority_list();

// Duplicated with consts in the other svc crates,
// but declaring them separately here to avoid adding another dependency.
// Declared as consts for reuse across reserve-v2 feature

const SANCTUM_SPL_SVC: &str = "sspUE1vrh7xRoXxGsg7vR1zde2WdGtJRbyK9uRumBDy";
const SANCTUM_SPL_MULTI_SVC: &str = "ssmbu3KZxgonUtjEMCKspZzxvUQCxAFnyh1rcHUeEDo";
const SPL_SVC: &str = "sp1V4h2gWorkGhVcazBc22Hfo2f5sd7jcjT4EDPrWFF";
const LIDO_SVC: &str = "1idUSy4MGGKyKhvjSnGZ6Zc7Q4eKQcibym4BkEEw9KR";
const MARINADE_SVC: &str = "mare3SCyfZkAndpBRBeonETmkCCB3TJTTrz8ZN2dnhP";
const WSOL_SVC: &str = "wsoGmxQLSvwWpuaidCApxN5kEowLe2HLQLJhCQnj4bE";

#[cfg(not(feature = "reserve-v2"))]
mod svc_whitelist {
    use super::*;

    /// Hardcoded whitelisted SOL value calculator program IDs
    #[generic_array_struct(all pub)]
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct SvcWhitelist<T> {
        pub sanctum_spl_svc: T,
        pub sanctum_spl_multi_svc: T,
        pub spl_svc: T,
        pub lido_svc: T,
        pub marinade_svc: T,
        pub wsol_svc: T,
    }

    pub const WHITELISTED_SVC_PROGS_STR: SvcWhitelist<&'static str> =
        SvcWhitelist::const_from_destr(SvcWhitelistDestr {
            sanctum_spl_svc: SANCTUM_SPL_SVC,
            sanctum_spl_multi_svc: SANCTUM_SPL_MULTI_SVC,
            spl_svc: SPL_SVC,
            lido_svc: LIDO_SVC,
            marinade_svc: MARINADE_SVC,
            wsol_svc: WSOL_SVC,
        });
}

#[cfg(feature = "reserve-v2")]
mod svc_whitelist {
    use super::*;

    /// Hardcoded whitelisted SOL value calculator program IDs
    #[generic_array_struct(all pub)]
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct SvcWhitelist<T> {
        pub sanctum_spl_svc: T,
        pub sanctum_spl_multi_svc: T,
        pub spl_svc: T,
        pub lido_svc: T,
        pub marinade_svc: T,
        pub wsol_svc: T,
        pub inf_svc: T,
    }

    pub const WHITELISTED_SVC_PROGS_STR: SvcWhitelist<&'static str> =
        SvcWhitelist::const_from_destr(SvcWhitelistDestr {
            sanctum_spl_svc: SANCTUM_SPL_SVC,
            sanctum_spl_multi_svc: SANCTUM_SPL_MULTI_SVC,
            spl_svc: SPL_SVC,
            lido_svc: LIDO_SVC,
            marinade_svc: MARINADE_SVC,
            wsol_svc: WSOL_SVC,
            inf_svc: "1nf7dspGYz1CTALJbtNgjvcSYWiFz5N3c2EuUZLSWCL",
        });
}

pub use svc_whitelist::*;

pub const WHITELISTED_SVC_PROGS: SvcWhitelist<[u8; 32]> = SvcWhitelist(const_map!(
    [0; 32],
    WHITELISTED_SVC_PROGS_STR.0,
    const_crypto::bs58::decode_pubkey
));

#[cfg(test)]
mod tests {
    use expect_test::expect;
    use solana_pubkey::Pubkey;

    use super::*;

    #[test]
    fn whitelisted_svcs_snapshot() {
        let all: String = WHITELISTED_SVC_PROGS
            .0
            .iter()
            .flat_map(|pk| [Pubkey::new_from_array(*pk).to_string(), ",\n".to_owned()])
            .collect();
        let exp = if cfg!(feature = "reserve-v2") {
            expect![[r#"
                sspUE1vrh7xRoXxGsg7vR1zde2WdGtJRbyK9uRumBDy,
                ssmbu3KZxgonUtjEMCKspZzxvUQCxAFnyh1rcHUeEDo,
                sp1V4h2gWorkGhVcazBc22Hfo2f5sd7jcjT4EDPrWFF,
                1idUSy4MGGKyKhvjSnGZ6Zc7Q4eKQcibym4BkEEw9KR,
                mare3SCyfZkAndpBRBeonETmkCCB3TJTTrz8ZN2dnhP,
                wsoGmxQLSvwWpuaidCApxN5kEowLe2HLQLJhCQnj4bE,
                1nf7dspGYz1CTALJbtNgjvcSYWiFz5N3c2EuUZLSWCL,
            "#]]
        } else {
            expect![[r#"
                sspUE1vrh7xRoXxGsg7vR1zde2WdGtJRbyK9uRumBDy,
                ssmbu3KZxgonUtjEMCKspZzxvUQCxAFnyh1rcHUeEDo,
                sp1V4h2gWorkGhVcazBc22Hfo2f5sd7jcjT4EDPrWFF,
                1idUSy4MGGKyKhvjSnGZ6Zc7Q4eKQcibym4BkEEw9KR,
                mare3SCyfZkAndpBRBeonETmkCCB3TJTTrz8ZN2dnhP,
                wsoGmxQLSvwWpuaidCApxN5kEowLe2HLQLJhCQnj4bE,
            "#]]
        };

        exp.assert_eq(&all);
    }
}
