//! In-memory `ReserveV2Pricing` for the reserve-v2 pricing program
//! (`uppoVuoFZuXisHkrxCU96VvNibU6vzxkEpeH3WbmnEn`).
//!
//! The program resolves each swap to one of two fee shapes via
//! [`inf1_pp_reserve_v2_core::route::classify_route`]:
//!
//! - [`ReserveV2SwapKind::Flat`](inf1_pp_reserve_v2_core::route::ReserveV2SwapKind::Flat):
//!   a flat input/output fee from the two mints' [`FeeEntry`]s.
//! - [`ReserveV2SwapKind::RangeOut`](inf1_pp_reserve_v2_core::route::ReserveV2SwapKind::RangeOut):
//!   a fee that depends on the pool's release-yield-adjusted SOL value and its
//!   wSOL reserves balance.
//!
//! Quoting the Flat route needs only the pricing state's account data. Quoting
//! the RangeOut route also needs the controller pool state, the pool's wSOL
//! reserves, and a slot -- so [`ReserveV2Pricing`] holds all of them and
//! [`ReserveV2Pricing::accounts_to_update_*`] returns the keys to fetch.
//!
//! [`FeeEntry`]: inf1_pp_reserve_v2_core::typedefs::FeeEntry
//! [`ReserveV2Pricing::accounts_to_update_*`]: inf1_pp_std::update::AccountsToUpdatePriceExactIn

use std::{convert::Infallible, error::Error, fmt::Display};

use inf1_ctl_core::{accounts::pool_state::PoolStateV2, err::Inf1CtlErr};
use inf1_pp_reserve_v2_core::{
    accounts::pricing_state_of_acc_data_packed,
    errs::ReserveV2ProgramErr,
    pda::CONST_PDA_KEYS_OWNED,
    pricing::{range_out_inputs, RangeOutInputsErr, ReserveV2SwapPricing},
    route::{classify_route, ReserveV2SwapKind},
    typedefs::{FeeEntryPackedList, MintNotFoundErr},
};
use inf1_pp_std::pair::Pair;

pub mod traits;
pub mod update;

// Re-exports
pub use inf1_pp_reserve_v2_core::*;

/// The clock sysvar. Not exposed by a `no_std` core crate, and each calculator
/// std crate pulls it from its own SDK (`solido_legacy_core`,
/// `sanctum_spl_stake_pool_core`), so restate it here.
pub(crate) const SYSVAR_CLOCK: [u8; 32] =
    const_crypto::bs58::decode_pubkey("SysvarC1ock11111111111111111111111111111111");

/// The offset of the `u64` amount in an SPL token account.
const TOKEN_ACC_AMOUNT_OFFSET: usize = 64;

/// The offset of the `u64` slot in the clock sysvar.
const CLOCK_ACC_SLOT_OFFSET: usize = 0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReserveV2PricingColErr {
    Program(ReserveV2ProgramErr),
    /// Reading the controller pool state and applying release-yield lookahead
    /// failed.
    Ctl(Inf1CtlErr),
    /// A RangeOut quote was asked for before the pool state, wSOL reserves, and
    /// clock had been fetched. Call an `update_*` first.
    NotUpdated,
}

impl Display for ReserveV2PricingColErr {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Program(e) => Display::fmt(e, f),
            Self::Ctl(e) => Display::fmt(e, f),
            Self::NotUpdated => f.write_str("ReserveV2Pricing has not been updated"),
        }
    }
}

impl Error for ReserveV2PricingColErr {}

impl From<ReserveV2ProgramErr> for ReserveV2PricingColErr {
    #[inline]
    fn from(e: ReserveV2ProgramErr) -> Self {
        Self::Program(e)
    }
}

impl From<MintNotFoundErr> for ReserveV2PricingColErr {
    #[inline]
    fn from(e: MintNotFoundErr) -> Self {
        Self::Program(ReserveV2ProgramErr::MintNotFound(e))
    }
}

impl From<RangeOutInputsErr> for ReserveV2PricingColErr {
    #[inline]
    fn from(e: RangeOutInputsErr) -> Self {
        match e {
            RangeOutInputsErr::Ctl(e) => Self::Ctl(e),
            RangeOutInputsErr::Program(e) => Self::Program(e),
        }
    }
}

impl From<Infallible> for ReserveV2PricingColErr {
    #[inline]
    fn from(_e: Infallible) -> Self {
        unreachable!()
    }
}

/// A snapshot of everything the reserve-v2 pricing program reads to quote a
/// swap: its own pricing state, the controller pool state, the pool's wSOL
/// reserves balance, and the current slot.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ReserveV2Pricing {
    /// Pricing state account data.
    pricing_state: Box<[u8]>,

    /// Only read by the RangeOut route.
    pool_state: Option<PoolStateV2>,
    wsol_balance: Option<u64>,
    curr_slot: Option<u64>,
}

impl ReserveV2Pricing {
    /// The fee entries of the pricing state. Empty if the account is not a
    /// valid pricing state, which is also what an empty account yields.
    #[inline]
    pub fn entries(&self) -> FeeEntryPackedList<'_> {
        match pricing_state_of_acc_data_packed(&self.pricing_state) {
            Some((_, entries)) => entries,
            None => FeeEntryPackedList::new(&[]),
        }
    }

    /// The pricing to apply to a swap between `mints`.
    ///
    /// Resolves the route first; only the RangeOut route reads the pool
    /// snapshot, so a flat swap quotes without any update beyond the pricing
    /// state.
    #[inline]
    pub fn swap_pricing_for(
        &self,
        Pair { inp, out }: &Pair<&[u8; 32]>,
    ) -> Result<ReserveV2SwapPricing, ReserveV2PricingColErr> {
        let route = classify_route(inp, out)?;

        let entries = self.entries();
        let input_entry = entries.find_by_mint(inp)?.into_fee_entry();
        let output_entry = entries.find_by_mint(out)?.into_fee_entry();

        let (pool_sol_value, wsol_balance) = match route {
            ReserveV2SwapKind::Flat(()) => (0, 0),
            ReserveV2SwapKind::RangeOut(()) => self.range_inputs()?,
        };

        Ok(ReserveV2SwapPricing::from_entries(
            route,
            &input_entry,
            &output_entry,
            pool_sol_value,
            wsol_balance,
        ))
    }

    /// `(pool_sol_value, wsol_balance)` as the program's `range_out_pricing`
    /// computes them, via the same core helper the program calls.
    fn range_inputs(&self) -> Result<(u64, u64), ReserveV2PricingColErr> {
        let ps = self
            .pool_state
            .as_ref()
            .ok_or(ReserveV2PricingColErr::NotUpdated)?;
        let curr_slot = self.curr_slot.ok_or(ReserveV2PricingColErr::NotUpdated)?;
        let wsol_balance = self
            .wsol_balance
            .ok_or(ReserveV2PricingColErr::NotUpdated)?;

        range_out_inputs(ps, curr_slot, wsol_balance).map_err(Into::into)
    }
}

/// The 3 const PDAs of the reserve-v2 pricing program plus the clock sysvar.
pub(crate) const PRICING_ACCOUNTS: [[u8; 32]; 4] = [
    *CONST_PDA_KEYS_OWNED.pricing_state(),
    *CONST_PDA_KEYS_OWNED.pool_state(),
    *CONST_PDA_KEYS_OWNED.wsol_reserves(),
    SYSVAR_CLOCK,
];

/// Reads the `u64` at `OFFSET`, or `None` if the account is too short.
#[inline]
fn u64_at<const OFFSET: usize>(data: &[u8]) -> Option<u64> {
    data.get(OFFSET..OFFSET + 8)
        .map(|s| u64::from_le_bytes(s.try_into().unwrap()))
}

/// The wSOL reserves ATA is a plain SPL token account; only its amount matters.
#[inline]
pub(crate) fn token_account_amount(data: &[u8]) -> Option<u64> {
    u64_at::<TOKEN_ACC_AMOUNT_OFFSET>(data)
}

/// The clock sysvar's slot is its first field.
#[inline]
pub(crate) fn clock_slot(data: &[u8]) -> Option<u64> {
    u64_at::<CLOCK_ACC_SLOT_OFFSET>(data)
}

#[cfg(test)]
mod tests {
    use inf1_pp_reserve_v2_core::{
        keys::CONST_KEYS_OWNED,
        typedefs::{FeeEntry, FeeEntryNanos, FeeEntryNanosDestr},
    };

    use super::*;

    fn entry(mint: [u8; 32], base: u32, output: u32) -> FeeEntry {
        FeeEntry {
            mint,
            threshold_nanos: 1,
            fee_nanos: FeeEntryNanos::from_destr(FeeEntryNanosDestr {
                base_fee: base,
                threshold_fee: base,
                max_fee: base,
                output_fee: output,
            }),
        }
    }

    fn pricing_state_acc_data(entries: &[FeeEntry]) -> Vec<u8> {
        let mut sorted = entries.to_vec();
        sorted.sort_by_key(|e| e.mint);
        let mut v = vec![0u8; 32];
        for e in sorted {
            v.extend_from_slice(e.into_fee_entry_packed().as_acc_data_arr());
        }
        v
    }

    fn pool_state(total_sol_value: u64) -> PoolStateV2 {
        let mut ps = PoolStateV2::init(0, [7u8; 32]);
        ps.total_sol_value = total_sol_value;
        ps.withheld_lamports = 0;
        ps.protocol_fee_lamports = 0;
        ps.last_release_slot = 100;
        ps
    }

    fn pricing(lst: [u8; 32]) -> ReserveV2Pricing {
        let lp = *CONST_KEYS_OWNED.lp_mint();
        let wsol = *CONST_KEYS_OWNED.wsol_mint();
        ReserveV2Pricing {
            pricing_state: pricing_state_acc_data(&[
                entry(lp, 0, 0),
                entry(wsol, 0, 0),
                entry(lst, 0, 0),
            ])
            .into(),
            pool_state: Some(pool_state(1_000_000)),
            wsol_balance: Some(400_000),
            curr_slot: Some(100),
        }
    }

    /// An LST -> LST swap is the Flat route; the pool snapshot must not affect
    /// it.
    #[test]
    fn flat_route_quotes_without_pool_state() {
        let lp = *CONST_KEYS_OWNED.lp_mint();
        let p = ReserveV2Pricing {
            pricing_state: pricing_state_acc_data(&[
                entry(lp, 0, 0),
                entry([1u8; 32], 0, 0),
                entry([2u8; 32], 0, 0),
            ])
            .into(),
            ..Default::default()
        };
        let got = p
            .swap_pricing_for(&Pair {
                inp: &[1u8; 32],
                out: &[2u8; 32],
            })
            .unwrap();
        assert!(matches!(got, ReserveV2SwapPricing::Flat(_)), "{got:?}");
    }

    /// An LST -> LP swap is the RangeOut route and reads the pool snapshot.
    /// The returned pricing must equal what the program builds from the same
    /// numbers.
    #[test]
    fn range_route_uses_pool_state() {
        let lp = *CONST_KEYS_OWNED.lp_mint();
        let p = pricing([1u8; 32]);
        let got = p
            .swap_pricing_for(&Pair {
                inp: &[1u8; 32],
                out: &lp,
            })
            .unwrap();
        let want = ReserveV2SwapPricing::from_entries(
            inf1_pp_reserve_v2_core::route::ReserveV2SwapKind::RangeOut(()),
            &entry([1u8; 32], 0, 0),
            &entry(lp, 0, 0),
            1_000_000,
            400_000,
        );
        assert_eq!(got, want);
    }

    /// The RangeOut route must not silently quote off a stale snapshot.
    #[test]
    fn range_route_requires_an_update() {
        let lp = *CONST_KEYS_OWNED.lp_mint();
        let mut p = pricing([1u8; 32]);
        p.pool_state = None;
        assert_eq!(
            p.swap_pricing_for(&Pair {
                inp: &[1u8; 32],
                out: &lp,
            }),
            Err(ReserveV2PricingColErr::NotUpdated)
        );
    }

    /// An unknown mint is the same error the program returns.
    #[test]
    fn unknown_mint_is_a_program_error() {
        let lp = *CONST_KEYS_OWNED.lp_mint();
        let p = pricing([1u8; 32]);
        let e = p
            .swap_pricing_for(&Pair {
                inp: &[9u8; 32],
                out: &lp,
            })
            .unwrap_err();
        assert!(matches!(e, ReserveV2PricingColErr::Program(_)), "{e:?}");
    }
}
