use solana_account::Account;
use solana_pubkey::Pubkey;

mod program;
mod slab;
mod spl_stake_pool;
mod system;
mod sysvars;
mod token;

pub use program::*;
pub use slab::*;
pub use spl_stake_pool::*;
pub use system::*;
pub use sysvars::*;
pub use token::*;

pub type PkAccountTup = (Pubkey, Account);

pub fn upsert_account(existing: &mut Vec<PkAccountTup>, new: PkAccountTup) {
    match existing.iter_mut().find(|(pk, _)| *pk == new.0) {
        Some(e) => e.1 = new.1,
        None => {
            existing.push(new);
        }
    }
}
