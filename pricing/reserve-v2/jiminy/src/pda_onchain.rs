use inf1_pp_reserve_v2_core::pda::{CONST_PDA_BUMPS, PRICING_STATE_SEED};
use jiminy_pda::PdaSigner;

/// # Note on SBF toolchain issue
///
/// Keeping individual `PdaSigner`s as separate constants via `const_1seed_signer` instead of
/// using a const generic-array-struct aggregate like the other consts because doing that
/// seems to trigger a miscompile bug in the onchain program where
///
/// `invoke_signed(..., &[*CONST_PDA_SIGNERS.pool_state()])`
///
/// works but
///
/// `invoke_signed(..., core::slice::from_ref(CONST_PDA_SIGNERS.pool_state()))`
///
/// fails with `signer privilege escalation`.
///
/// Possibly because the first one copies the `PdaSeed`s data from .rodata onto the stack
/// and then passes pointer to that stack data to invoke_signed while the latter passes a
/// pointer to .rodata directly and the sol_invoke_signed_c syscall has some issues with the latter.
///
/// Or maybe its a lifetime issue with jiminy-cpi
///
/// Whatever the issue, im not dealing with it right now
///
/// Update: it's likely ye olde SBF relocations bug that corrupts const pointers
macro_rules! const_1seed_signer {
    ($NAME:ident, $seed:expr, $bump_ref:expr) => {
        pub const $NAME: PdaSigner = {
            // Copy bump value instead of referencing const field addr
            // due to issue documented in above comment
            const BUMP_VAL: u8 = *$bump_ref;
            const BUMP_ARR: [u8; 1] = [BUMP_VAL];
            PdaSigner::new(&[PdaSeed::new($seed.as_slice()), PdaSeed::new(&BUMP_ARR)])
        };
    };
}

use jiminy_pda::PdaSeed;

const_1seed_signer!(
    PRICING_STATE_SIGNER,
    PRICING_STATE_SEED,
    CONST_PDA_BUMPS.pricing_state()
);
