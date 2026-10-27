pub const WSOL_BRIDGE_PDA_PRE: u8 = b'x';

pub const PORTFOLIO_PDA_PRE: u8 = b'y';

pub const PROTOCOL_PDA_SEED: u8 = b'a';

pub const fn portfolio_pda_seeds(sols_mint: &[u8; 32]) -> (&[u8; 1], &[u8; 32]) {
    (&[PORTFOLIO_PDA_PRE], sols_mint)
}

pub const fn protocol_pda_seeds() -> &'static [u8; 1] {
    &[PROTOCOL_PDA_SEED]
}

pub const fn wsol_bridge_pda_seeds(sols_mint: &[u8; 32]) -> (&[u8; 1], &[u8; 32]) {
    (&[WSOL_BRIDGE_PDA_PRE], sols_mint)
}
