use inf1_ctl_core::{keys::POOL_STATE_BUMP, pda::POOL_STATE_SEED};
use jiminy_pda::{PdaSeed, PdaSigner};

pub const POOL_SEED_SIGNER: PdaSigner = PdaSigner::new(&[
    PdaSeed::new(POOL_STATE_SEED.as_slice()),
    PdaSeed::new(&[POOL_STATE_BUMP]),
]);
