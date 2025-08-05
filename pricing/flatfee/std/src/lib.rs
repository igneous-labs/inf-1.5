use core::{borrow::Borrow, hash::Hash};
use std::collections::HashMap;

use inf1_pp_flatfee_core::{accounts::fee::FeeAccount, pda::fee_account_seeds};

// Re-exports
pub use inf1_pp_flatfee_core::*;

pub mod traits;

pub type FindPdaFnPtr = fn(&[&[u8]], &[u8; 32]) -> Option<([u8; 32], u8)>;

pub type CreatePdaFnPtr = fn(&[&[u8]], &[u8; 32]) -> Option<[u8; 32]>;

pub type FlatFeePricingStd = FlatFeePricing<FindPdaFnPtr, CreatePdaFnPtr>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlatFeePricing<F, C> {
    /// `None` when acc not yet fetched
    lp_withdrawal_fee_bps: Option<u16>,

    /// key=mint
    ///
    /// Entry does not exist if acc not yet fetched
    lsts: HashMap<[u8; 32], FeeAccount>,

    find_pda: F,

    create_pda: C,
}

/// Constructors
impl<
        F: Fn(&[&[u8]], &[u8; 32]) -> Option<([u8; 32], u8)>,
        C: Fn(&[&[u8]], &[u8; 32]) -> Option<[u8; 32]>,
    > FlatFeePricing<F, C>
{
    #[inline]
    pub const fn new(
        lp_withdrawal_fee_bps: Option<u16>,
        lsts: HashMap<[u8; 32], FeeAccount>,
        find_pda_fn: F,
        create_pda_fn: C,
    ) -> Self {
        Self {
            lp_withdrawal_fee_bps,
            lsts,
            find_pda: find_pda_fn,
            create_pda: create_pda_fn,
        }
    }
}

/// Accounts to update 1
impl<F, C> FlatFeePricing<F, C> {
    #[inline]
    pub const fn account_to_update_remove_liquidity(&self) -> [u8; 32] {
        inf1_pp_flatfee_core::keys::STATE_ID
    }
}

/// Accounts to update 2
impl<
        F: Fn(&[&[u8]], &[u8; 32]) -> Option<([u8; 32], u8)>,
        C: Fn(&[&[u8]], &[u8; 32]) -> Option<[u8; 32]>,
    > FlatFeePricing<F, C>
{
    #[inline]
    pub fn accounts_to_update_swap<'a, I: IntoIterator<Item = &'a [u8; 32]>>(
        &self,
        mints: I,
    ) -> impl Iterator<Item = [u8; 32]> + use<'a, '_, I, F, C> {
        mints.into_iter().map(|mint| self.fee_account_pda(mint))
    }
}

/// Update
impl<F, C> FlatFeePricing<F, C> {
    #[inline]
    pub fn upsert_fee_account(&mut self, mint: [u8; 32], fee_account: FeeAccount) {
        self.lsts.insert(mint, fee_account);
    }

    #[inline]
    pub const fn update_lp_withdrawal_fee_bps(&mut self, lp_withdrawal_fee_bps: u16) {
        self.lp_withdrawal_fee_bps = Some(lp_withdrawal_fee_bps);
    }
}

/// Getters
impl<F, C> FlatFeePricing<F, C> {
    #[inline]
    pub fn fee_account<Q>(&self, mint: &Q) -> Option<&FeeAccount>
    where
        [u8; 32]: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.lsts.get(mint)
    }

    #[inline]
    pub const fn lp_withdrawal_fee_bps(&self) -> Option<u16> {
        self.lp_withdrawal_fee_bps
    }
}

/// PDA
impl<
        F: Fn(&[&[u8]], &[u8; 32]) -> Option<([u8; 32], u8)>,
        C: Fn(&[&[u8]], &[u8; 32]) -> Option<[u8; 32]>,
    > FlatFeePricing<F, C>
{
    #[inline]
    pub fn fee_account_pda(&self, mint: &[u8; 32]) -> [u8; 32] {
        let (s1, s2) = fee_account_seeds(mint);

        self.lsts.get(mint).map_or_else(
            // unwrap-safety: fee accounts should all be of valid, found PDAs
            || (self.find_pda)(&[s1, s2], &ID).unwrap().0,
            // unwrap-safety: fee accounts should have valid bumps
            |FeeAccount { bump, .. }| {
                (self.create_pda)(&[s1, s2, core::slice::from_ref(bump)], &ID).unwrap()
            },
        )
    }
}
