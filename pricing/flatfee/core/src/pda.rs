pub const STATE_SEED: [u8; 5] = *b"state";

pub const FEE_ACCOUNT_SEED: [u8; 3] = *b"fee";

pub const fn fee_account_seeds(lst_mint: &[u8; 32]) -> (&[u8; 3], &[u8; 32]) {
    (&FEE_ACCOUNT_SEED, lst_mint)
}
