use const_crypto::{
    bs58::{encode_pubkey, Base58Str},
    ed25519::derive_program_address,
};
use generic_array_struct::generic_array_struct;

use crate::{internal_utils::const_map, keys::CONST_KEYS_OWNED, token_info::TokenInfo};

pub const POOL_STATE_SEED: [u8; 5] = *b"state";

pub const LST_STATE_LIST_SEED: [u8; 14] = *b"lst-state-list";

pub const PROTOCOL_FEE_SEED: [u8; 12] = *b"protocol-fee";

pub const REBALANCE_RECORD_SEED: [u8; 16] = *b"rebalance-record";

pub const DISABLE_POOL_AUTHORITY_LIST_SEED: [u8; 27] = *b"disable-pool-authority-list";

pub const fn const_find_pool_state(prog_id: &[u8; 32]) -> ([u8; 32], u8) {
    derive_program_address(&[&POOL_STATE_SEED], prog_id)
}

pub const fn const_find_lst_state_list(prog_id: &[u8; 32]) -> ([u8; 32], u8) {
    derive_program_address(&[&LST_STATE_LIST_SEED], prog_id)
}

pub const fn const_find_protocol_fee(prog_id: &[u8; 32]) -> ([u8; 32], u8) {
    derive_program_address(&[&PROTOCOL_FEE_SEED], prog_id)
}

pub const fn const_find_rebalance_record(prog_id: &[u8; 32]) -> ([u8; 32], u8) {
    derive_program_address(&[&REBALANCE_RECORD_SEED], prog_id)
}

pub const fn const_find_disable_pool_authority_list(prog_id: &[u8; 32]) -> ([u8; 32], u8) {
    derive_program_address(&[&DISABLE_POOL_AUTHORITY_LIST_SEED], prog_id)
}

#[generic_array_struct(all pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConstPdas<T> {
    pub pool_state: T,
    pub lst_state_list: T,
    pub protocol_fee: T,
    pub rebalance_record: T,
    pub disable_pool_authority_list: T,
}

pub const CONST_PDAS: ConstPdas<([u8; 32], u8)> = ConstPdas::const_from_destr(ConstPdasDestr {
    pool_state: const_find_pool_state(CONST_KEYS_OWNED.program()),
    lst_state_list: const_find_lst_state_list(CONST_KEYS_OWNED.program()),
    protocol_fee: const_find_protocol_fee(CONST_KEYS_OWNED.program()),
    rebalance_record: const_find_rebalance_record(CONST_KEYS_OWNED.program()),
    disable_pool_authority_list: const_find_disable_pool_authority_list(CONST_KEYS_OWNED.program()),
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

/// PDA seeds to use with ATA program to find pool reserves ATA
pub const fn pool_reserves_ata_seeds<'a>(token: &TokenInfo<&'a [u8; 32]>) -> [&'a [u8; 32]; 3] {
    ata_seeds(CONST_PDA_KEYS_OWNED.pool_state(), token)
}

/// PDA seeds to use with ATA program to find protocol fee accumulator ATA
pub const fn protocol_fee_accumulator_ata_seeds<'a>(
    token: &TokenInfo<&'a [u8; 32]>,
) -> [&'a [u8; 32]; 3] {
    ata_seeds(CONST_PDA_KEYS_OWNED.protocol_fee(), token)
}

/// PDA seeds to use with ATA program to find ATA addr
///
/// ## Params
/// - `auth`: `POOL_STATE` for pool reserves, `PROTOCOL_FEE` for protocol fee accumulator
pub const fn ata_seeds<'a>(auth: &'a [u8; 32], t: &TokenInfo<&'a [u8; 32]>) -> [&'a [u8; 32]; 3] {
    [auth, t.program(), t.mint()]
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use super::*;

    #[test]
    fn const_pda_snapshots() {
        let (expect_strs, expect_bumps) = if cfg!(feature = "reserve-v2") {
            (
                expect![[r#"
                ConstPdas(
                    [
                        "9zMRqtjkTvUm4kVtz2MrPiJnr9spUmYsr8Uqis7y3Brv",
                        "Brg7vhSTVp76eTy3xjwRgBUfh711eH61io2Xvqj72UA5",
                        "8HikxZiJ6VUYitYKhkqUhEs4YS3vbztDKrZMVrC6i6Hx",
                        "DZx5xErD3sW4ugtQDbM1c2N3fTsxogPLk2F25nqhmuSe",
                        "7ri7XRq5SLE6EcYJcjJ8Cn1KX279sJGVZvRdSsdt5hWr",
                    ],
                )
            "#]],
                expect![[r#"
                ConstPdas(
                    [
                        254,
                        255,
                        255,
                        254,
                        255,
                    ],
                )
            "#]],
            )
        } else {
            (
                expect![[r#"
                ConstPdas(
                    [
                        "AYhux5gJzCoeoc1PoJ1VxwPDe22RwcvpHviLDD1oCGvW",
                        "Gb7m4daakbVbrFLR33FKMDVMHAprRZ66CSYt4bpFwUgS",
                        "6U8Ve7NuTVq9pb3xEC2ZwxBhceWULUuJn1nSKCTraq5r",
                        "GVoB1QdoqCzdSsQr7zsxyGZB1HhWpfejm6ZZduvseSNa",
                        "FJc6b3iyYaD5p24aKQ2FcM7WVATapPGq65LhY1MDKXzG",
                    ],
                )
            "#]],
                expect![[r#"
                ConstPdas(
                    [
                        255,
                        255,
                        255,
                        255,
                        255,
                    ],
                )
            "#]],
            )
        };
        expect_strs.assert_debug_eq(&CONST_PDA_KEY_STRS);
        expect_bumps.assert_debug_eq(&CONST_PDA_BUMPS);
    }
}
