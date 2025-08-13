pub const SLAB_SEED: [u8; 4] = *b"slab";

pub const fn slab_seeds() -> [&'static [u8; 4]; 1] {
    [&SLAB_SEED]
}
