use const_crypto::bs58::decode_pubkey;
use generic_array_struct::generic_array_struct;

#[generic_array_struct(all pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConstKeys<T> {
    pub lp_mint: T,
    pub wsol_mint: T,
}

pub const WSOL_MINT_STR: &str = "So11111111111111111111111111111111111111112";

pub const CONST_KEYS: ConstKeys<[u8; 32]> = ConstKeys::const_from_destr(ConstKeysDestr {
    // TODO: Placeholder for Reserve V2 controller deployment
    lp_mint: [1; 32],
    wsol_mint: decode_pubkey(WSOL_MINT_STR),
});

pub const LP_MINT: [u8; 32] = *CONST_KEYS.lp_mint();
pub const WSOL_MINT: [u8; 32] = *CONST_KEYS.wsol_mint();
