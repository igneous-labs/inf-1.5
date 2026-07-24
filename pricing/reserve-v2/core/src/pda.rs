use const_crypto::{
    bs58::{encode_pubkey, Base58Str},
    ed25519::derive_program_address,
};
use generic_array_struct::generic_array_struct;

use crate::internal_utils::const_map;
use crate::keys::CONST_KEYS_OWNED;

pub const PRICING_STATE_SEED: [u8; 1] = *b"p";
pub const POOL_STATE_SEED: [u8; 5] = *b"state";

pub const fn const_find_pricing_state(prog_id: &[u8; 32]) -> ([u8; 32], u8) {
    derive_program_address(&[&PRICING_STATE_SEED], prog_id)
}

pub const fn const_find_pool_state(prog_id: &[u8; 32]) -> ([u8; 32], u8) {
    derive_program_address(&[&POOL_STATE_SEED], prog_id)
}

pub const fn ata_seeds<'a>(
    auth: &'a [u8; 32],
    token_prog: &'a [u8; 32],
    mint: &'a [u8; 32],
) -> [&'a [u8; 32]; 3] {
    [auth, token_prog, mint]
}

pub const fn wsol_reserves_ata_seeds(pool_state: &[u8; 32]) -> [&[u8; 32]; 3] {
    ata_seeds(
        pool_state,
        CONST_KEYS_OWNED.tokenkeg(),
        CONST_KEYS_OWNED.wsol_mint(),
    )
}

pub const fn const_find_wsol_reserves_ata(pool_state: &[u8; 32]) -> ([u8; 32], u8) {
    let [s0, s1, s2] = wsol_reserves_ata_seeds(pool_state);
    derive_program_address(
        &[s0.as_slice(), s1.as_slice(), s2.as_slice()],
        CONST_KEYS_OWNED.atoken(),
    )
}

#[generic_array_struct(all pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConstPdas<T> {
    pub pricing_state: T,
    pub pool_state: T,
    pub wsol_reserves: T,
}

const POOL_STATE: ([u8; 32], u8) = const_find_pool_state(CONST_KEYS_OWNED.reserve_v2_program());

pub const CONST_PDAS: ConstPdas<([u8; 32], u8)> = ConstPdas::const_from_destr(ConstPdasDestr {
    pricing_state: const_find_pricing_state(CONST_KEYS_OWNED.program()),
    pool_state: POOL_STATE,
    wsol_reserves: const_find_wsol_reserves_ata(&POOL_STATE.0),
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

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use super::*;

    #[test]
    fn const_pda_snapshots() {
        let expect_strs = expect![[r#"
            ConstPdas(
                [
                    "8mvhAnVbCMyRU9mn9uH9ZwEeJGchZvBbHHecqHemtwRZ",
                    "9zMRqtjkTvUm4kVtz2MrPiJnr9spUmYsr8Uqis7y3Brv",
                    "6nBpYJ3oeraht4cFyFPj3TLFpNuy8SRMQA2KGRXVAEHY",
                ],
            )
        "#]];
        let expect_bumps = expect![[r#"
            ConstPdas(
                [
                    255,
                    254,
                    254,
                ],
            )
        "#]];
        expect_strs.assert_debug_eq(&CONST_PDA_KEY_STRS);
        expect_bumps.assert_debug_eq(&CONST_PDA_BUMPS);
    }
}
