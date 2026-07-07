use inf1_svc_generic::pda::STATE_SEED;
use jiminy_pda::PdaSeed;

pub fn state_signer(bump: &u8) -> [PdaSeed<'_>; 2] {
    [
        PdaSeed::new(&STATE_SEED),
        PdaSeed::new(core::slice::from_ref(bump)),
    ]
}
