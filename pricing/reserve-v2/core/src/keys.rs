use const_crypto::bs58::decode_pubkey;
use generic_array_struct::generic_array_struct;

use crate::internal_utils::const_map;

#[generic_array_struct(all pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConstAccs<T> {
    pub program: T,
    pub init_admin: T,
    pub lp_mint: T,
    pub wsol_mint: T,
}

pub const CONST_KEY_STRS: ConstAccs<&'static str> = ConstAccs::const_from_destr(ConstAccsDestr {
    program: "uppoVuoFZuXisHkrxCU96VvNibU6vzxkEpeH3WbmnEn",
    init_admin: "GRwm4EXMyVwtftQeTft7DZT3HBRxx439PrKq4oM6BwoZ",
    // TODO: Placeholder for Reserve V2 controller deployment
    lp_mint: "4vJ9JU1bJJE96FWSJKvHsmmFADCg4gpZQff4P3bkLKi",
    wsol_mint: "So11111111111111111111111111111111111111112",
});

pub const CONST_KEYS_OWNED: ConstAccs<[u8; 32]> =
    ConstAccs(const_map!([0; 32], CONST_KEY_STRS.0, decode_pubkey));
