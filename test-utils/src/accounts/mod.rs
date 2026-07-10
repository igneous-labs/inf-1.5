use std::collections::HashMap;

use solana_account::Account;
use solana_pubkey::Pubkey;

mod controller;
mod gen_svc;
mod pp;
mod program;
mod svc;
mod system;
mod sysvars;
mod token;

pub use controller::*;
pub use gen_svc::*;
pub use pp::*;
pub use program::*;
pub use svc::*;
pub use system::*;
pub use sysvars::*;
pub use token::*;

pub type AccountMap = HashMap<Pubkey, Account>;
