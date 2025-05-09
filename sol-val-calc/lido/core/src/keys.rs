use inf1_svc_generic::pda::const_find_state;

macro_rules! id_str {
    ($ID_STR:ident, $ID:ident, $pkstr:expr) => {
        pub const $ID_STR: &str = $pkstr;
        pub const $ID: [u8; 32] = const_crypto::bs58::decode_pubkey($ID_STR);
    };
}
pub(crate) use id_str;

id_str!(
    POOL_PROG_ID_STR,
    POOL_PROG_ID,
    "CrX7kMhLC3cSsXJdT7JDgqrRVWGnUpX3gfEfxxU2NVLi"
);

id_str!(
    POOL_PROGDATA_ID_STR,
    POOL_PROGDATA_ID,
    "CHZNLhDXKrsXBmmv947RFciquwBsn2NdABmhpxoX3wgZ"
);

const STATE: ([u8; 32], u8) = const_find_state(&crate::ID);
pub const STATE_ID: [u8; 32] = STATE.0;
pub const STATE_BUMP: u8 = STATE.1;
pub const STATE_ID_STR: &str = const_crypto::bs58::encode_pubkey(&STATE_ID).str();
