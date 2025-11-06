use std::{borrow::Borrow, collections::HashSet, ops::RangeInclusive};

use inf1_ctl_core::keys::SYS_PROG_ID;
use jiminy_sysvar_rent::Rent;
use proptest::{collection::vec, prelude::*};
use solana_account::Account;
use solana_pubkey::Pubkey;

use crate::{assert_list_changes, Diff, ListChange, ListChanges};

pub fn disable_pool_auth_list_account(pks: Vec<[u8; 32]>) -> Account {
    let data: Vec<u8> = pks.into_iter().flatten().collect();
    let (lamports, owner) = if data.is_empty() {
        // Empty account owned by system program
        (0, Pubkey::new_from_array(SYS_PROG_ID))
    } else {
        (
            Rent::DEFAULT.min_balance(data.len()),
            Pubkey::new_from_array(inf1_ctl_core::ID),
        )
    };
    Account {
        lamports,
        data,
        owner,
        executable: false,
        rent_epoch: u64::MAX,
    }
}

pub fn any_disable_pool_auth_list(
    len: RangeInclusive<usize>,
) -> impl Strategy<Value = Vec<[u8; 32]>> {
    vec(any::<[u8; 32]>(), len).prop_map(|mut v| {
        // use hashset to dedup instead of sort + dedup
        // because disable pool auth list is not always in sorted order
        let mut dedup = HashSet::new();
        // insert returns true if did not previously contain value
        v.retain(|pk| dedup.insert(*pk));
        v
    })
}

pub type DisablePoolAuthListChange = ListChange<Diff<[u8; 32]>, [u8; 32]>;

pub fn assert_diffs_disable_pool_auth_list(
    changes: impl IntoIterator<Item = impl Borrow<DisablePoolAuthListChange>>,
    bef: impl IntoIterator<Item = impl Borrow<[u8; 32]>>,
    aft: impl IntoIterator<Item = impl Borrow<[u8; 32]>>,
) {
    assert_list_changes(changes, bef, aft, Diff::assert);
}

pub type DisablePoolAuthListChanges<'a> = ListChanges<'a, Diff<[u8; 32]>, [u8; 32]>;

impl DisablePoolAuthListChanges<'_> {
    fn idx_by_pk(&self, pk: &[u8; 32]) -> usize {
        self.list.iter().position(|l| l == pk).unwrap()
    }

    pub fn with_del_by_pk(self, pk: &[u8; 32]) -> Self {
        let i = self.idx_by_pk(pk);
        self.with_del(i)
    }

    pub fn with_diff_by_pk(self, pk: &[u8; 32], diff: Diff<[u8; 32]>) -> Self {
        let i = self.idx_by_pk(pk);
        self.with_diff(i, diff)
    }
}

/// A valid DisablePoolAuthList:
/// - should not have duplicate entries
pub fn assert_valid_disable_pool_auth_list(l: &[[u8; 32]]) {
    let mut dedup = HashSet::new();
    l.iter().for_each(|pk| {
        // insert returns false if already present in set
        if !dedup.insert(pk) {
            panic!("duplicate pk {}", Pubkey::new_from_array(*pk));
        }
    });
}
