use std::{
    error::Error,
    fmt::{Display, Formatter},
    iter::{once, Once},
};

use inf1_pp_flatslab_core::{accounts::Slab, keys::SLAB_ID};
use inf1_pp_std::{
    pair::Pair,
    update::{
        Account, AccountsToUpdateAll, AccountsToUpdateMintLp, AccountsToUpdatePriceExactIn,
        AccountsToUpdatePriceExactOut, AccountsToUpdateRedeemLp, UpdateErr, UpdateMap,
        UpdatePricingProg,
    },
};

use crate::FlatSlabPricing;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FlatSlabPricingUpdateErr {
    AccDeser { pk: [u8; 32] },
}

impl Display for FlatSlabPricingUpdateErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::AccDeser { .. } => f.write_str("AccDeser"),
        }
    }
}

impl Error for FlatSlabPricingUpdateErr {}

pub type PkIter = Once<[u8; 32]>;

impl FlatSlabPricing {
    #[inline]
    pub fn accs_to_update(&self) -> PkIter {
        once(SLAB_ID)
    }

    #[inline]
    pub fn update_slab(
        &mut self,
        update_map: impl UpdateMap,
    ) -> Result<(), UpdateErr<FlatSlabPricingUpdateErr>> {
        let slab = update_map.get_account_checked(&SLAB_ID)?;
        if Slab::of_acc_data(slab.data()).is_none() {
            return Err(UpdateErr::Inner(FlatSlabPricingUpdateErr::AccDeser {
                pk: SLAB_ID,
            }));
        }

        self.slab_acc_data = slab.data().into();

        Ok(())
    }
}

// Accounts

impl AccountsToUpdateAll for FlatSlabPricing {
    type PkIter = PkIter;

    #[inline]
    fn accounts_to_update_all(
        &self,
        _all_mints: impl IntoIterator<Item = [u8; 32]>,
    ) -> Self::PkIter {
        self.accs_to_update()
    }
}

impl AccountsToUpdatePriceExactIn for FlatSlabPricing {
    type PkIter = PkIter;

    #[inline]
    fn accounts_to_update_price_exact_in(&self, _swap_mints: &Pair<&[u8; 32]>) -> Self::PkIter {
        self.accs_to_update()
    }
}

impl AccountsToUpdatePriceExactOut for FlatSlabPricing {
    type PkIter = PkIter;

    #[inline]
    fn accounts_to_update_price_exact_out(&self, _swap_mints: &Pair<&[u8; 32]>) -> Self::PkIter {
        self.accs_to_update()
    }
}

impl AccountsToUpdateMintLp for FlatSlabPricing {
    type PkIter = PkIter;

    #[inline]
    fn accounts_to_update_mint_lp(&self, _inp_mint: &[u8; 32]) -> Self::PkIter {
        self.accs_to_update()
    }
}

impl AccountsToUpdateRedeemLp for FlatSlabPricing {
    type PkIter = PkIter;

    #[inline]
    fn accounts_to_update_redeem_lp(&self, _out_mint: &[u8; 32]) -> Self::PkIter {
        self.accs_to_update()
    }
}

// Update

impl UpdatePricingProg for FlatSlabPricing {
    type InnerErr = FlatSlabPricingUpdateErr;

    #[inline]
    fn update_mint_lp(
        &mut self,
        _inp_mint: &[u8; 32],
        update_map: impl UpdateMap,
    ) -> Result<(), UpdateErr<Self::InnerErr>> {
        self.update_slab(update_map)
    }

    #[inline]
    fn update_redeem_lp(
        &mut self,
        _out_mint: &[u8; 32],
        update_map: impl UpdateMap,
    ) -> Result<(), UpdateErr<Self::InnerErr>> {
        self.update_slab(update_map)
    }

    fn update_price_exact_in(
        &mut self,
        _swap_mints: &Pair<&[u8; 32]>,
        update_map: impl UpdateMap,
    ) -> Result<(), UpdateErr<Self::InnerErr>> {
        self.update_slab(update_map)
    }

    fn update_price_exact_out(
        &mut self,
        _swap_mints: &Pair<&[u8; 32]>,
        update_map: impl UpdateMap,
    ) -> Result<(), UpdateErr<Self::InnerErr>> {
        self.update_slab(update_map)
    }

    fn update_all(
        &mut self,
        _all_mints: impl IntoIterator<Item = [u8; 32]>,
        update_map: impl UpdateMap,
    ) -> Result<(), UpdateErr<Self::InnerErr>> {
        self.update_slab(update_map)
    }
}
