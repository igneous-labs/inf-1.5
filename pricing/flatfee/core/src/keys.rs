use crate::pda::STATE_SEED;

macro_rules! id_str {
    ($ID_STR:ident, $ID:ident, $pkstr:expr) => {
        pub const $ID_STR: &str = $pkstr;
        pub const $ID: [u8; 32] = const_crypto::bs58::decode_pubkey($ID_STR);
    };
}
pub(crate) use id_str;

macro_rules! const_pda {
    ($INTER:ident, $ID_STR:ident, $ID:ident, $BUMP:ident, $seeds:expr) => {
        const $INTER: ([u8; 32], u8) =
            const_crypto::ed25519::derive_program_address($seeds, &crate::ID);
        pub const $ID: [u8; 32] = $INTER.0;
        pub const $BUMP: u8 = $INTER.1;
        pub const $ID_STR: &str = const_crypto::bs58::encode_pubkey(&$ID).str();
    };
}

const_pda!(
    STATE,
    STATE_ID_STR,
    STATE_ID,
    STATE_BUMP,
    &[STATE_SEED.as_slice()]
);
