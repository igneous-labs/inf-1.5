use const_crypto::{
    bs58::{encode_pubkey, Base58Str},
    ed25519::derive_program_address,
};
use generic_array_struct::generic_array_struct;

use crate::internal_utils::const_map;
use crate::keys::CONST_KEYS_OWNED;

pub const PRICING_STATE_SEED: [u8; 1] = *b"p";

pub const fn const_find_pricing_state(prog_id: &[u8; 32]) -> ([u8; 32], u8) {
    derive_program_address(&[&PRICING_STATE_SEED], prog_id)
}

#[generic_array_struct(all pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConstPdas<T> {
    pub pricing_state: T,
}

pub const CONST_PDAS: ConstPdas<([u8; 32], u8)> = ConstPdas::const_from_destr(ConstPdasDestr {
    pricing_state: const_find_pricing_state(CONST_KEYS_OWNED.program()),
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
                ],
            )
        "#]];
        let expect_bumps = expect![[r#"
            ConstPdas(
                [
                    255,
                ],
            )
        "#]];
        expect_strs.assert_debug_eq(&CONST_PDA_KEY_STRS);
        expect_bumps.assert_debug_eq(&CONST_PDA_BUMPS);
    }
}
