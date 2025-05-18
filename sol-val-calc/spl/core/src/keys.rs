macro_rules! id_str {
    ($ID_STR:ident, $ID:ident, $pkstr:expr) => {
        pub const $ID_STR: &str = $pkstr;
        pub const $ID: [u8; 32] = const_crypto::bs58::decode_pubkey($ID_STR);
    };
}
pub(crate) use id_str;

macro_rules! const_state_pda {
    () => {
        const STATE: ([u8; 32], u8) = inf1_svc_generic::pda::const_find_state(&ID);
        pub const STATE_ID: [u8; 32] = STATE.0;
        pub const STATE_BUMP: u8 = STATE.1;
        pub const STATE_ID_STR: &str = const_crypto::bs58::encode_pubkey(&STATE_ID).str();
    };
}

pub mod spl {
    use super::*;

    id_str!(ID_STR, ID, "sp1V4h2gWorkGhVcazBc22Hfo2f5sd7jcjT4EDPrWFF");
    id_str!(
        POOL_PROG_ID_STR,
        POOL_PROG_ID,
        "SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy"
    );
    id_str!(
        POOL_PROGDATA_ID_STR,
        POOL_PROGDATA_ID,
        "EmiU8AQkB2sswTxVB6aCmsAJftoowZGGDXuytm6X65R3"
    );
    const_state_pda!();
}

pub mod sanctum_spl {
    use super::*;

    id_str!(ID_STR, ID, "sspUE1vrh7xRoXxGsg7vR1zde2WdGtJRbyK9uRumBDy");
    id_str!(
        POOL_PROG_ID_STR,
        POOL_PROG_ID,
        "SP12tWFxD9oJsVWNavTTBZvMbA6gkAmxtVgxdqvyvhY"
    );
    id_str!(
        POOL_PROGDATA_ID_STR,
        POOL_PROGDATA_ID,
        "Cn5fegqLh8Fmvffisr4Wk3LmuaUgMMzTFfEuidpZFsvV"
    );
    const_state_pda!();
}

pub mod sanctum_spl_multi {
    use super::*;

    id_str!(ID_STR, ID, "ssmbu3KZxgonUtjEMCKspZzxvUQCxAFnyh1rcHUeEDo");
    id_str!(
        POOL_PROG_ID_STR,
        POOL_PROG_ID,
        "SPMBzsVUuoHA4Jm6KunbsotaahvVikZs1JyTW6iJvbn"
    );
    id_str!(
        POOL_PROGDATA_ID_STR,
        POOL_PROGDATA_ID,
        "HxBTMuB7cFBPVWVJjTi9iBF8MPd7mfY1QnrrWfLAySFd"
    );
    const_state_pda!();
}
