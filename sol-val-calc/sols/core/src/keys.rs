use inf1_svc_generic::{
    keys::{ConstAccs, ConstAccsDestr},
    pda::ConstPdas,
};

pub const ID_STR: &str = "sssQe6fXL4KRDGeGvoFULZakZjwQ1DKd7vu29QDJBxP";
pub const ID: [u8; 32] = *CONST_KEYS_OWNED.program();

pub const CONST_KEY_STRS: ConstAccs<&str> = ConstAccs::const_from_destr(ConstAccsDestr {
    program: ID_STR,
    pool_prog: sanctum_sols_core::keys::CONST_KEY_STRS.program_id(),
    init_manager: "7eYqVNDg6kkDaWwNkGnMgQX9Vsn4yuc9zyx2t5n6AG5p",
});

pub const CONST_KEYS_OWNED: ConstAccs<[u8; 32]> = CONST_KEY_STRS.const_keys();

pub const CONST_PDAS: ConstPdas<([u8; 32], u8)> =
    ConstPdas::const_find_from_const_accs(&CONST_KEYS_OWNED);
