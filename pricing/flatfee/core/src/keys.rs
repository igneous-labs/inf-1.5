use crate::{pda::STATE_SEED, ID};

macro_rules! id_str {
    // variant 1: generate from base58 string
    ($ID_STR:ident, $ID:ident, $pkstr:expr) => {
        pub const $ID_STR: &str = $pkstr;
        pub const $ID: [u8; 32] = const_crypto::bs58::decode_pubkey($ID_STR);
    };

    // variant 2: $ID has already been found (e.g. a PDA)
    ($ID_STR:ident, $ID:ident) => {
        pub const $ID_STR: &str = const_crypto::bs58::encode_pubkey(&$ID).str();
    };
}
use const_crypto::ed25519::derive_program_address;
pub(crate) use id_str;

const STATE: ([u8; 32], u8) = derive_program_address(&[STATE_SEED.as_slice()], &ID);
pub const STATE_ID: [u8; 32] = STATE.0;
pub const STATE_BUMP: u8 = STATE.1;
id_str!(STATE_ID_STR, STATE_ID);
