use const_crypto::bs58::decode_pubkey;

/// TODO: Placeholder for Reserve V2 controller deployment
pub const LP_MINT: [u8; 32] = [1; 32];

pub const WSOL_MINT_STR: &str = "So11111111111111111111111111111111111111112";
pub const WSOL_MINT: [u8; 32] = decode_pubkey(WSOL_MINT_STR);
