use generic_array_struct::generic_array_struct;

use crate::internal_utils::const_map;

#[generic_array_struct(all pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConstKeys<T> {
    pub lp_mint: T,
    pub wsol_mint: T,
}

pub const CONST_KEY_STRS: ConstKeys<&'static str> = ConstKeys::const_from_destr(ConstKeysDestr {
    // TODO: Random key placeholder for Reserve V2 controller deployment
    lp_mint: "GHkxQo8rFu67e1yM1cwhAiHoHvsMkYgQayetFD2ZFDY5",
    wsol_mint: "So11111111111111111111111111111111111111112",
});

pub const CONST_KEYS_OWNED: ConstKeys<[u8; 32]> = ConstKeys(const_map!(
    [0; 32],
    CONST_KEY_STRS.0,
    const_crypto::bs58::decode_pubkey
));
