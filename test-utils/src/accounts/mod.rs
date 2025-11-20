use std::collections::HashMap;

use solana_account::Account;
use solana_pubkey::Pubkey;

mod program;
mod spl_stake_pool;
mod system;
mod sysvars;
mod token;

pub use program::*;
pub use spl_stake_pool::*;
pub use system::*;
pub use sysvars::*;
pub use token::*;

pub type AccountMap = HashMap<Pubkey, Account>;
