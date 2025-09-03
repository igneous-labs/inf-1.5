mod program;
mod sysvars;

pub use program::*;
use solana_account::Account;
use solana_pubkey::Pubkey;
pub use sysvars::*;

pub type PkAccountTup = (Pubkey, Account);
