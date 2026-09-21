use std::{
    array,
    error::Error,
    fmt::{Display, Formatter},
};

use inf1_ctl_core::{accounts::pool_state::VerPoolState, typedefs::versioned::V1_2};
use inf1_pp_reserve_v2_core::accounts::pricing_state_of_acc_data_packed;
use inf1_pp_std::{
    pair::Pair,
    update::{
        Account, AccountsToUpdateAll, AccountsToUpdateMintLp, AccountsToUpdatePriceExactIn,
        AccountsToUpdatePriceExactOut, AccountsToUpdateRedeemLp, UpdateErr, UpdateMap,
        UpdatePricingProg,
    },
};

use crate::{clock_slot, token_account_amount, ReserveV2Pricing, PRICING_ACCOUNTS};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReserveV2PricingUpdateErr {
    AccDeser { pk: [u8; 32] },
}

impl Display for ReserveV2PricingUpdateErr {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AccDeser { pk } => write!(f, "AccDeser {pk:?}"),
        }
    }
}

impl Error for ReserveV2PricingUpdateErr {}

/// The pricing state PDA, the controller pool state PDA, the pool's wSOL
/// reserves ATA, and the clock sysvar. All four are const, so the set is the
/// same for every update procedure.
pub type PkIter = array::IntoIter<[u8; 32], 4>;

impl ReserveV2Pricing {
    /// Refresh every field from freshly fetched accounts.
    ///
    /// The pricing state is always needed; the pool state, wSOL reserves and
    /// clock are only read when quoting a RangeOut route, but they are fetched
    /// unconditionally so the struct is always quoteable.
    pub fn update_accounts(
        &mut self,
        update_map: impl UpdateMap,
    ) -> Result<(), UpdateErr<ReserveV2PricingUpdateErr>> {
        let [pricing_state_pk, pool_state_pk, wsol_reserves_pk, clock_pk] = PRICING_ACCOUNTS;

        let acc = update_map.get_account_checked(&pricing_state_pk)?;
        if pricing_state_of_acc_data_packed(acc.data()).is_none() {
            return Err(UpdateErr::Inner(ReserveV2PricingUpdateErr::AccDeser {
                pk: pricing_state_pk,
            }));
        }
        self.pricing_state = acc.data().into();

        let acc = update_map.get_account_checked(&pool_state_pk)?;
        let pool_state = match VerPoolState::try_from_acc_data(acc.data()) {
            Some(V1_2::V2(ps)) => ps,
            _ => {
                return Err(UpdateErr::Inner(ReserveV2PricingUpdateErr::AccDeser {
                    pk: pool_state_pk,
                }));
            }
        };
        self.pool_state = Some(pool_state);

        let acc = update_map.get_account_checked(&wsol_reserves_pk)?;
        self.wsol_balance = Some(token_account_amount(acc.data()).ok_or(UpdateErr::Inner(
            ReserveV2PricingUpdateErr::AccDeser {
                pk: wsol_reserves_pk,
            },
        ))?);

        let acc = update_map.get_account_checked(&clock_pk)?;
        self.curr_slot = Some(clock_slot(acc.data()).ok_or(UpdateErr::Inner(
            ReserveV2PricingUpdateErr::AccDeser { pk: clock_pk },
        ))?);

        Ok(())
    }
}

// Accounts

impl AccountsToUpdateAll for ReserveV2Pricing {
    type PkIter = PkIter;

    #[inline]
    fn accounts_to_update_all(
        &self,
        _all_mints: impl IntoIterator<Item = [u8; 32]>,
    ) -> Self::PkIter {
        PRICING_ACCOUNTS.into_iter()
    }
}

impl AccountsToUpdatePriceExactIn for ReserveV2Pricing {
    type PkIter = PkIter;

    #[inline]
    fn accounts_to_update_price_exact_in(&self, _swap_mints: &Pair<&[u8; 32]>) -> Self::PkIter {
        PRICING_ACCOUNTS.into_iter()
    }
}

impl AccountsToUpdatePriceExactOut for ReserveV2Pricing {
    type PkIter = PkIter;

    #[inline]
    fn accounts_to_update_price_exact_out(&self, _swap_mints: &Pair<&[u8; 32]>) -> Self::PkIter {
        PRICING_ACCOUNTS.into_iter()
    }
}

impl AccountsToUpdateMintLp for ReserveV2Pricing {
    type PkIter = PkIter;

    #[inline]
    fn accounts_to_update_mint_lp(&self, _inp_mint: &[u8; 32]) -> Self::PkIter {
        PRICING_ACCOUNTS.into_iter()
    }
}

impl AccountsToUpdateRedeemLp for ReserveV2Pricing {
    type PkIter = PkIter;

    #[inline]
    fn accounts_to_update_redeem_lp(&self, _out_mint: &[u8; 32]) -> Self::PkIter {
        PRICING_ACCOUNTS.into_iter()
    }
}

// Update

impl UpdatePricingProg for ReserveV2Pricing {
    type InnerErr = ReserveV2PricingUpdateErr;

    #[inline]
    fn update_mint_lp(
        &mut self,
        _inp_mint: &[u8; 32],
        update_map: impl UpdateMap,
    ) -> Result<(), UpdateErr<Self::InnerErr>> {
        self.update_accounts(update_map)
    }

    #[inline]
    fn update_redeem_lp(
        &mut self,
        _out_mint: &[u8; 32],
        update_map: impl UpdateMap,
    ) -> Result<(), UpdateErr<Self::InnerErr>> {
        self.update_accounts(update_map)
    }

    #[inline]
    fn update_price_exact_in(
        &mut self,
        _swap_mints: &Pair<&[u8; 32]>,
        update_map: impl UpdateMap,
    ) -> Result<(), UpdateErr<Self::InnerErr>> {
        self.update_accounts(update_map)
    }

    #[inline]
    fn update_price_exact_out(
        &mut self,
        _swap_mints: &Pair<&[u8; 32]>,
        update_map: impl UpdateMap,
    ) -> Result<(), UpdateErr<Self::InnerErr>> {
        self.update_accounts(update_map)
    }

    #[inline]
    fn update_all(
        &mut self,
        _all_mints: impl IntoIterator<Item = [u8; 32]>,
        update_map: impl UpdateMap,
    ) -> Result<(), UpdateErr<Self::InnerErr>> {
        self.update_accounts(update_map)
    }
}
