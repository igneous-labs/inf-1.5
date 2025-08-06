use std::{
    error::Error,
    fmt::{Display, Formatter},
};

use inf1_pp_core::pair::Pair;
use inf1_pp_flatfee_core::accounts::fee::FeeAccountPacked;
use inf1_pp_std::update::{Account, UpdateErr, UpdateMap};

use crate::FlatFeePricing;

pub type SwapPkIter = std::array::IntoIter<[u8; 32], 2>;

impl<
        F: Fn(&[&[u8]], &[u8; 32]) -> Option<([u8; 32], u8)>,
        C: Fn(&[&[u8]], &[u8; 32]) -> Option<[u8; 32]>,
    > FlatFeePricing<F, C>
{
    #[inline]
    pub fn accounts_to_update_swap_pair(&self, Pair { inp, out }: &Pair<&[u8; 32]>) -> SwapPkIter {
        [inp, out]
            .map(|mint| self.fee_account_pda(mint))
            .into_iter()
    }

    #[inline]
    pub fn update_swap_pair(
        &mut self,
        Pair { inp, out }: &Pair<&[u8; 32]>,
        update_map: impl UpdateMap,
    ) -> Result<(), UpdateErr<FlatFeePricingUpdateErr>> {
        [inp, out].into_iter().try_for_each(|mint| {
            let fee_acc = self.fee_account_pda(mint);
            let new_fee_acc = update_map.get_account_checked(&fee_acc)?;
            let new_fee_acc = FeeAccountPacked::of_acc_data(new_fee_acc.data())
                .ok_or(UpdateErr::Inner(FlatFeePricingUpdateErr::AccDeser {
                    pk: fee_acc,
                }))?
                .into_fee_account();

            self.upsert_fee_account(**mint, new_fee_acc);

            Ok(())
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FlatFeePricingUpdateErr {
    AccDeser { pk: [u8; 32] },
}

impl Display for FlatFeePricingUpdateErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::AccDeser { .. } => f.write_str("AccDeser"),
        }
    }
}

impl Error for FlatFeePricingUpdateErr {}
