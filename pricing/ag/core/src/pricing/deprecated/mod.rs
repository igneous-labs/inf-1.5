#![deprecated(
    since = "0.2.0",
    note = "Use SwapExactIn/Out with out_mint=LP token (INF) instead"
)]
#![allow(deprecated)]

mod mint;
mod redeem;

pub use mint::*;
pub use redeem::*;
