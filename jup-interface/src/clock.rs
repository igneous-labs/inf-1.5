//! Utils for handling special-casing of sysvar Clock account

use inf1_std::inf1_svc_ag_std::inf1_svc_marinade_core::sanctum_marinade_liquid_staking_core::MSOL_MINT_ADDR;

use crate::consts::{INF_MINT_ADDR, WSOL_MINT_ADDR};

pub const fn is_epoch_affected_lst_mint(mint: &[u8; 32]) -> bool {
    match *mint {
        INF_MINT_ADDR | MSOL_MINT_ADDR | WSOL_MINT_ADDR => false,
        // stsol, spls
        _ => true,
    }
}
