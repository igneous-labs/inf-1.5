use std::cmp::Ordering;

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

pub type PkAccountTup = (Pubkey, Account);

pub fn upsert_account(existing: &mut Vec<PkAccountTup>, new: PkAccountTup) {
    match existing.iter_mut().find(|(pk, _)| *pk == new.0) {
        Some(e) => e.1 = new.1,
        None => {
            existing.push(new);
        }
    }
}

/// Dedups account entries that have the same pubkey.
///
/// Breaks ties using desc order of account owner, so that
/// system accounts are removed by dedup first.
pub fn dedup_accounts(v: &mut Vec<PkAccountTup>) {
    v.sort_by(|a, b| match a.0.cmp(&b.0) {
        Ordering::Equal => b.1.owner.cmp(&a.1.owner),
        o => o,
    });
    v.dedup_by_key(|(pk, _)| *pk);
}
