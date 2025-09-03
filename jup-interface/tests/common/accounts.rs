use jupiter_amm_interface::{AmmContext, ClockRef};
use lazy_static::lazy_static;
use solana_pubkey::Pubkey;

pub const JUPSOL_MINT_ADDR: [u8; 32] =
    Pubkey::from_str_const("jupSoLaHXQiZZTSfEWMTRRgpnyFm8f6sZdosWBjx93v").to_bytes();

lazy_static! {
    pub static ref AMM_CONTEXT: AmmContext = {
        AmmContext {
            clock_ref: ClockRef::default(),
        }
    };
}
