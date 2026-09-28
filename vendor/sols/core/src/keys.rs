use const_crypto::{
    bs58::{decode_pubkey, encode_pubkey, Base58Str},
    ed25519::derive_program_address,
};
use generic_array_struct::generic_array_struct;

use crate::{internal_utils::const_map, pda::protocol_pda_seeds};

#[generic_array_struct(all pub)]
pub struct ConstAccs<T> {
    pub program_id: T,
    pub tokenkeg_prog: T,
    pub wsol_mint: T,
    pub assoc_token: T,
    pub system_prog: T,
    pub sysvar_instructions: T,
    pub sysvar_rent: T,
    pub init_protocol_fee_controller: T,
    pub init_protocol_beneficiary: T,
    pub init_svc_whitelist_auth: T,
}

pub const CONST_KEY_STRS: ConstAccs<&'static str> = ConstAccs::const_from_destr(ConstAccsDestr {
    program_id: "so1f7APRw5pJ5iNJrM9g5X9tQgXzk8kMUnXxDNdt99b",
    tokenkeg_prog: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
    wsol_mint: "So11111111111111111111111111111111111111112",
    assoc_token: "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL",
    system_prog: "11111111111111111111111111111111",
    sysvar_instructions: "Sysvar1nstructions1111111111111111111111111",
    sysvar_rent: "SysvarRent111111111111111111111111111111111",
    init_protocol_fee_controller: "8JS6XsMPo2u3EyeeY3p2jvzHEhdUtCKgarHpJ3PAonyv",
    init_protocol_beneficiary: "8JS6XsMPo2u3EyeeY3p2jvzHEhdUtCKgarHpJ3PAonyv",
    init_svc_whitelist_auth: "8JS6XsMPo2u3EyeeY3p2jvzHEhdUtCKgarHpJ3PAonyv",
});

pub const CONST_KEYS_OWNED: ConstAccs<[u8; 32]> =
    ConstAccs(const_map!([0; 32], CONST_KEY_STRS.0, decode_pubkey));

#[generic_array_struct(all pub)]
pub struct ConstPdas<T> {
    pub protocol: T,
}

const fn const_find_protocol_pda(prog_id: &[u8; 32]) -> ([u8; 32], u8) {
    let s = protocol_pda_seeds();
    derive_program_address(&[s], prog_id)
}

pub const CONST_PDAS: ConstPdas<([u8; 32], u8)> = ConstPdas::const_from_destr(ConstPdasDestr {
    protocol: const_find_protocol_pda(CONST_KEYS_OWNED.program_id()),
});

const fn const_pda_addr((pda, _): &([u8; 32], u8)) -> [u8; 32] {
    *pda
}
pub const CONST_PDA_KEYS_OWNED: ConstPdas<[u8; 32]> =
    ConstPdas(const_map!([0; 32], CONST_PDAS.0, const_pda_addr));

const fn const_pda_bump((_, bump): &([u8; 32], u8)) -> u8 {
    *bump
}
pub const CONST_PDA_BUMPS: ConstPdas<u8> = ConstPdas(const_map!(0, CONST_PDAS.0, const_pda_bump));

const fn const_pda_base58str(pda_addr: &[u8; 32]) -> Base58Str {
    encode_pubkey(pda_addr)
}
const CONST_PDA_BASE58STRS: ConstPdas<Base58Str> = ConstPdas(const_map!(
    encode_pubkey(&[0; 32]),
    CONST_PDA_KEYS_OWNED.0,
    const_pda_base58str
));

const fn const_base58_to_str(base58str: &Base58Str) -> &str {
    base58str.str()
}
pub const CONST_PDA_KEY_STRS: ConstPdas<&'static str> =
    ConstPdas(const_map!("", CONST_PDA_BASE58STRS.0, const_base58_to_str));
