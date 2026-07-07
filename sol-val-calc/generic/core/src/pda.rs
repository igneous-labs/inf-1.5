use const_crypto::{
    bs58::{decode_pubkey, encode_pubkey, Base58Str},
    ed25519::derive_program_address,
};
use generic_array_struct::generic_array_struct;

use crate::keys::{const_map, GLOBAL_CONST_KEYS_OWNED};

/// Program-specific const PDAs
#[generic_array_struct(all pub)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConstPdas<T> {
    /// Singleton state PDA
    pub state: T,

    /// The stake pool program's BPF loader V3 program data PDA account
    pub pool_progdata: T,
}

impl ConstPdas<&str> {
    pub const fn const_keys(self) -> ConstPdas<[u8; 32]> {
        ConstPdas(const_map!([0; 32], self.0, decode_pubkey))
    }
}

impl ConstPdas<([u8; 32], u8)> {
    /// # Params
    ///
    /// Not to be confused with their result
    ///
    /// - state: SOL value calculator program addr
    /// - pool_progdata: stake pool program addr
    pub const fn const_find(params: &ConstPdas<[u8; 32]>) -> Self {
        Self::const_from_destr(ConstPdasDestr {
            state: const_find_state(params.state()),
            pool_progdata: const_find_programdata(params.pool_progdata()),
        })
    }

    pub const fn const_keys(self) -> ConstPdas<[u8; 32]> {
        const fn f((k, _): &([u8; 32], u8)) -> [u8; 32] {
            *k
        }
        ConstPdas(const_map!([0; 32], self.0, f))
    }

    pub const fn const_bumps(self) -> ConstPdas<u8> {
        const fn f((_, b): &([u8; 32], u8)) -> u8 {
            *b
        }
        ConstPdas(const_map!(0, self.0, f))
    }
}

impl ConstPdas<[u8; 32]> {
    pub const fn const_key_base58strs(self) -> ConstPdas<Base58Str> {
        ConstPdas(const_map!(encode_pubkey(&[0; 32]), self.0, encode_pubkey))
    }
}

impl ConstPdas<Base58Str> {
    pub const fn const_key_strs(&self) -> ConstPdas<&str> {
        const fn f(base58str: &Base58Str) -> &str {
            base58str.str()
        }
        ConstPdas(const_map!("", self.0, f))
    }
}

pub const STATE_SEED: [u8; 5] = *b"state";

pub const fn const_find_state(prog_id: &[u8; 32]) -> ([u8; 32], u8) {
    derive_program_address(&[&STATE_SEED], prog_id)
}

pub const fn const_find_programdata(prog_id: &[u8; 32]) -> ([u8; 32], u8) {
    derive_program_address(&[prog_id], GLOBAL_CONST_KEYS_OWNED.bpf_loader_v3())
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use super::*;

    #[test]
    fn sanctum_spl_multi_snapshot() {
        const PARAMS: ConstPdas<[u8; 32]> = ConstPdas::const_from_destr(ConstPdasDestr {
            state: "ssmbu3KZxgonUtjEMCKspZzxvUQCxAFnyh1rcHUeEDo",
            pool_progdata: "SPMBzsVUuoHA4Jm6KunbsotaahvVikZs1JyTW6iJvbn",
        })
        .const_keys();
        const PDAS: ConstPdas<([u8; 32], u8)> = ConstPdas::const_find(&PARAMS);
        const BUMPS: ConstPdas<u8> = PDAS.const_bumps();
        const KEY_STRS: ConstPdas<&str> = PDAS.const_keys().const_key_base58strs().const_key_strs();

        expect![[r#"
            ConstPdas(
                [
                    255,
                    255,
                ],
            )
        "#]]
        .assert_debug_eq(&BUMPS);
        expect![[r#"
            ConstPdas(
                [
                    "Ehcuy2BzuY9BscqcH2K43tDKqoi6xQHxChtVjzrMfvU8",
                    "HxBTMuB7cFBPVWVJjTi9iBF8MPd7mfY1QnrrWfLAySFd",
                ],
            )
        "#]]
        .assert_debug_eq(&KEY_STRS);
    }
}
