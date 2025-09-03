use crate::pda::{const_find_lst_state_list, const_find_pool_state, const_find_protocol_fee};

macro_rules! id_str {
    ($ID_STR:ident, $ID:ident, $pkstr:expr) => {
        pub const $ID_STR: &str = $pkstr;
        pub const $ID: [u8; 32] = const_crypto::bs58::decode_pubkey($ID_STR);
    };
}
pub(crate) use id_str;

id_str!(
    ATOKEN_ID_STR,
    ATOKEN_ID,
    "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"
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
